use rangemap::{RangeMap, RangeSet};
use std::collections::{BTreeMap, BTreeSet};
use walrus::ir::{Instr, InstrSeqId, InstrSeqType, LoadKind, Value};
use walrus::{
    ConstExpr, ExportItem, FunctionBuilder, FunctionId, FunctionKind, GlobalKind, InstrSeqBuilder,
    LocalFunction, LocalId, Module, ModuleFunctions, ModuleLocals, ModuleTypes, ValType,
};

fn main() {
    let mut module =
        Module::from_file("target/wasm32-unknown-unknown/release/wasm_calling_test.wasm").unwrap();

    dbg!(&module.locals.iter().collect::<Vec<_>>());

    let mut candidates = Vec::new();
    let mut supports = Vec::new();

    for export in module.exports.iter() {
        if export.name.starts_with("WRAPPED_") {
            candidates.push(export.clone());
        } else if export.name.starts_with("SUPPORT_") {
            supports.push(export.clone());
        }
    }

    let support_values = supports
        .iter()
        .map(|global| {let value = match global.item {
            ExportItem::Function(fid) => {
                let function = module.funcs.get(fid);

                match &function.kind {
                    FunctionKind::Local(local) => {
                        let block = local.block(local.entry_block());

                        match &block.instrs[0] {
                            (Instr::Const(v), _) => walrus_value_as_u64(&v.value),
                            (other, _) => panic!(
                                "A support function must consist of a constant value, not {other:?}"
                            ),
                        }
                    }
                    other => panic!("Bad function {other:?}"),
                }
            }
            ExportItem::Global(gid) => {
                let global = module.globals.get(gid);
                match &global.kind {
                    GlobalKind::Local(ConstExpr::Value(v)) => walrus_value_as_u64(v),
                    other => panic!("Bad global value {other:?}"),
                }
            }
            other => panic!("Bad global value {other:?}"),
        };
            (global.name.clone(), value)
        })
        .collect::<BTreeMap<_, _>>();

    dbg!(&candidates, &support_values);

    for export in supports {
        module.exports.delete(export.id());
    }
    for export in candidates {
        let function_id = match &export.item {
            ExportItem::Function(fid) => *fid,
            other => panic!("Export must be a function {other:?}"),
        };

        let identifier = &export.name["WRAPPED_".len()..];

        let mut inputs = BTreeMap::new();
        for i in 0usize.. {
            let prefix = format!("SUPPORT_{identifier}_INPUT_{i}_");

            if let Some((name, value)) = get_support_value(&support_values, &prefix, i) {
                inputs.insert(name, value);
            } else {
                break;
            }
        }
        let output = get_support_value(&support_values, &format!("SUPPORT_{identifier}_OUTPUT"), 0)
            .map(|(_, v)| v);

        dbg!(&inputs, &output);

        let Module {
            funcs,
            types,
            exports,
            locals,
            ..
        } = &mut module;

        let old_function = match &funcs.get(function_id).kind {
            FunctionKind::Local(local) => local,
            other => panic!("Transformed functions must be local {other:?}"),
        };

        let new_function_id = ScannedFunction::scan(old_function, locals, inputs, output)
            .build(identifier, types, funcs, locals);

        funcs.delete(function_id);
        exports.delete(export.id());
        exports.add(identifier, new_function_id);
    }

    module
        .emit_wasm_file("target/wasm32-unknown-unknown/release/wasm_calling_test_adjusted.wasm")
        .unwrap();
}

fn walrus_value_as_u64(v: &Value) -> u64 {
    match v {
        Value::I32(v) => *v as u64,
        Value::I64(v) => *v as u64,
        Value::F32(v) => *v as u64,
        Value::F64(v) => *v as u64,
        Value::V128(v) => *v as u64,
    }
}

#[derive(Clone, Debug)]
struct SupportValueData {
    pub address: u64,
    pub size: u64,

    pub index: usize,
}

fn get_support_value(
    support_values: &BTreeMap<String, u64>,
    prefix: &str,
    index: usize,
) -> Option<(String, SupportValueData)> {
    let relevant_values = support_values
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix));

    let mut name = None;
    let mut size = None;
    let mut address = None;

    for (key, value) in relevant_values {
        if key.ends_with("SIZE") {
            size = Some(*value);
        } else {
            name = Some(key[prefix.len()..].to_string());
            address = Some(*value);
        }
    }

    if let (Some(name), Some(size), Some(address)) = (name, size, address) {
        Some((
            name,
            SupportValueData {
                address,
                size,
                index,
            },
        ))
    } else {
        None
    }
}

#[derive(Debug)]
struct ScannedFunction {
    entry: InstrSeqId,
    blocks: BTreeMap<InstrSeqId, (InstrSeqType, Vec<Instr>)>,

    params: Vec<ValType>,

    results: Vec<ValType>,

    input_range: RangeMap<u64, usize>,
    output_range: RangeMap<u64, usize>,

    inputs: BTreeMap<(usize, u64), ValType>,
}

impl ScannedFunction {
    fn scan(
        source: &LocalFunction,
        locals: &ModuleLocals,

        inputs: BTreeMap<String, SupportValueData>,
        output: Option<SupportValueData>,
    ) -> Self {
        let mut input_range = RangeMap::new();
        for value in inputs.values() {
            input_range.insert(value.address..(value.address + value.size), value.index);
        }
        let mut output_range = RangeMap::new();
        for value in output.iter() {
            output_range.insert(value.address..(value.address + value.size), value.index);
        }

        let mut this = ScannedFunction {
            entry: source.entry_block(),
            blocks: BTreeMap::new(),

            params: Vec::new(),

            results: Vec::new(),

            input_range,
            output_range,

            inputs: BTreeMap::new(),
        };

        let mut block_ids = BTreeSet::new();
        let mut working_set = BTreeSet::from_iter([source.entry_block()]);

        while let Some(id) = working_set.pop_first() {
            if !block_ids.insert(id) {
                continue;
            }

            let block = source.block(source.entry_block());

            let mut instructions = Vec::with_capacity(block.instrs.len());

            for (instruction, _) in &block.instrs {
                instructions.push(instruction.clone());

                match instruction {
                    // All instructions that jump to a different block
                    Instr::Block(inner) => {
                        working_set.insert(inner.seq);
                    }
                    Instr::Loop(inner) => {
                        working_set.insert(inner.seq);
                    }
                    Instr::Br(inner) => {
                        working_set.insert(inner.block);
                    }
                    Instr::BrIf(inner) => {
                        working_set.insert(inner.block);
                    }
                    Instr::BrTable(inner) => {
                        for block in inner
                            .blocks
                            .iter()
                            .copied()
                            .chain(std::iter::once(inner.default))
                        {
                            working_set.insert(block);
                        }
                    }
                    Instr::BrOnCast(inner) => {
                        working_set.insert(inner.block);
                    }
                    Instr::BrOnCastFail(inner) => {
                        working_set.insert(inner.block);
                    }
                    Instr::BrOnNull(inner) => {
                        working_set.insert(inner.block);
                    }
                    Instr::BrOnNonNull(inner) => {
                        working_set.insert(inner.block);
                    }

                    // When it's a memory load, check if it is from the arguments
                    Instr::Load(inner) => {
                        if let Some(arg_index) = this.input_range.get(&inner.arg.offset).copied() {
                            let ty = match inner.kind {
                                LoadKind::I32 { .. }
                                | LoadKind::I32_8 { .. }
                                | LoadKind::I32_16 { .. } => ValType::I32,
                                LoadKind::I64 { .. }
                                | LoadKind::I64_8 { .. }
                                | LoadKind::I64_16 { .. }
                                | LoadKind::I64_32 { .. } => ValType::I64,

                                LoadKind::F32 { .. } => ValType::F32,
                                LoadKind::F64 { .. } => ValType::F64,

                                LoadKind::V128 { .. } => ValType::V128,
                            };

                            this.inputs.insert((arg_index, inner.arg.offset), ty);
                        }
                    }

                    _ => (),
                }
            }

            this.blocks.insert(id, (block.ty.clone(), instructions));
        }

        dbg!(&this.input_range, &this.output_range, &this.inputs);

        this
    }

    fn build(
        mut self,
        identifier: &str,
        types: &mut ModuleTypes,
        funcs: &mut ModuleFunctions,
        locals: &mut ModuleLocals,
    ) -> FunctionId {
        let args = self
            .params
            .iter()
            .map(|param| locals.add(*param))
            .collect::<Vec<_>>();

        let mut block_id_mapping = BTreeMap::new();

        let mut builder = FunctionBuilder::new(types, &self.params, &self.results);
        builder.name(identifier.to_string());

        for (in_block_id, (ty, _)) in self.blocks.iter().filter(|(key, _)| **key != self.entry) {
            let out_block_id = builder.dangling_instr_seq(ty.clone()).id();

            block_id_mapping.insert(*in_block_id, out_block_id);
        }

        let mut body = builder.func_body();

        block_id_mapping.insert(self.entry, body.id());

        self.build_block(self.entry, &mut body);

        builder.finish(args, funcs)
    }

    fn build_block(&mut self, id: InstrSeqId, builder: &mut InstrSeqBuilder) {
        let (_, instructions) = self.blocks.get(&id).unwrap();

        for instruction in instructions.clone() {
            match instruction {
                Instr::Block(..)
                | Instr::Loop(..)
                | Instr::Br(..)
                | Instr::BrIf(..)
                | Instr::BrTable(..) => (),
                default => {
                    builder.instr(default);
                }
            }
        }
    }
}
