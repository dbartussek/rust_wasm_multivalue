use std::collections::{BTreeMap, BTreeSet};
use walrus::{
    ExportItem, FunctionBuilder, FunctionId, FunctionKind, ImportId, LocalFunction, Module,
    ModuleFunctions, ModuleLocals, ModuleTypes, ValType,
    ir::{
        Block, Br, BrIf, BrOnCast, BrOnCastFail, BrOnNonNull, BrOnNull, BrTable, Instr, InstrSeqId,
        InstrSeqType, Loop,
    },
};

fn main() {
    let mut module =
        Module::from_file("target/wasm32-unknown-unknown/release/wasm_calling_test.wasm").unwrap();

    let support_imports: BTreeMap<ImportId, String> = module
        .imports
        .iter()
        .filter(|import| import.name.starts_with("wasm_calling_support"))
        .map(|import| (import.id(), import.name.clone()))
        .collect();

    let support_functions: BTreeMap<FunctionId, HelperFunctionDirection> = module
        .functions()
        .filter_map(|f| match &f.kind {
            FunctionKind::Import(def) => {
                if let Some(_) = support_imports.get(&def.import) {
                    let function_type = module.types.get(def.ty).as_function().unwrap();
                    let direction = function_type
                        .params()
                        .first()
                        .map(|output| HelperFunctionDirection::Output(*output))
                        .or_else(|| {
                            function_type
                                .results()
                                .first()
                                .map(|input| HelperFunctionDirection::Input(*input))
                        })
                        .unwrap();

                    Some((f.id(), direction))
                } else {
                    None
                }
            },
            _ => None,
        })
        .collect();

    for import in support_imports.keys() {
        module.imports.delete(*import);
    }
    for function in support_functions.keys() {
        module.funcs.delete(*function);
    }

    let mut candidates = Vec::new();

    for export in module.exports.iter() {
        if export.name.starts_with("WRAPPED_") {
            candidates.push(export.clone());
        }
    }

    for export in candidates {
        let function_id = match &export.item {
            ExportItem::Function(fid) => *fid,
            other => panic!("Export must be a function {other:?}"),
        };

        let identifier = &export.name["WRAPPED_".len()..];

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

        let new_function_id = ScannedFunction::scan(old_function, &support_functions).build(
            identifier,
            types,
            funcs,
            locals,
            &support_functions,
        );

        funcs.delete(function_id);
        exports.delete(export.id());
        exports.add(identifier, new_function_id);
    }

    module
        .emit_wasm_file("target/wasm32-unknown-unknown/release/wasm_calling_test_adjusted.wasm")
        .unwrap();
}

#[derive(Debug, Copy, Clone)]
enum HelperFunctionDirection {
    Input(ValType),
    Output(ValType),
}

#[derive(Debug)]
struct ScannedFunction {
    entry: InstrSeqId,
    blocks: BTreeMap<InstrSeqId, (InstrSeqType, Vec<Instr>)>,

    params: Vec<ValType>,
    results: Vec<ValType>,
}

impl ScannedFunction {
    fn scan(
        source: &LocalFunction,
        support_functions: &BTreeMap<FunctionId, HelperFunctionDirection>,
    ) -> Self {
        let mut this = ScannedFunction {
            entry: source.entry_block(),
            blocks: BTreeMap::new(),

            params: Vec::new(),
            results: Vec::new(),
        };

        let mut block_ids = BTreeSet::new();
        let mut working_set = BTreeSet::from_iter([source.entry_block()]);

        while let Some(id) = working_set.pop_first() {
            if !block_ids.insert(id) {
                continue;
            }

            let block = source.block(id);

            let mut instructions = Vec::with_capacity(block.instrs.len());

            for (instruction, _) in &block.instrs {
                instructions.push(instruction.clone());

                match instruction {
                    // All instructions that jump to a different block
                    Instr::Block(inner) => {
                        working_set.insert(inner.seq);
                    },
                    Instr::Loop(inner) => {
                        working_set.insert(inner.seq);
                    },
                    Instr::Br(inner) => {
                        working_set.insert(inner.block);
                    },
                    Instr::BrIf(inner) => {
                        working_set.insert(inner.block);
                    },
                    Instr::BrTable(inner) => {
                        for block in inner
                            .blocks
                            .iter()
                            .copied()
                            .chain(std::iter::once(inner.default))
                        {
                            working_set.insert(block);
                        }
                    },
                    Instr::BrOnCast(inner) => {
                        working_set.insert(inner.block);
                    },
                    Instr::BrOnCastFail(inner) => {
                        working_set.insert(inner.block);
                    },
                    Instr::BrOnNull(inner) => {
                        working_set.insert(inner.block);
                    },
                    Instr::BrOnNonNull(inner) => {
                        working_set.insert(inner.block);
                    },

                    Instr::Call(inner) => {
                        if let Some(helper) = support_functions.get(&inner.func) {
                            match helper {
                                HelperFunctionDirection::Input(ty) => {
                                    this.params.push(ty.clone());
                                },
                                HelperFunctionDirection::Output(ty) => {
                                    this.results.push(ty.clone());
                                },
                            }
                        }
                    },

                    _ => (),
                }
            }

            this.blocks.insert(id, (block.ty.clone(), instructions));
        }

        this
    }

    fn build(
        self,
        identifier: &str,
        types: &mut ModuleTypes,
        funcs: &mut ModuleFunctions,
        locals: &mut ModuleLocals,
        support_functions: &BTreeMap<FunctionId, HelperFunctionDirection>,
    ) -> FunctionId {
        let arg_locals = self
            .params
            .iter()
            .map(|param| locals.add(*param))
            .collect::<Vec<_>>();
        let results_locals = self
            .results
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

        block_id_mapping.insert(self.entry, builder.func_body_id());

        let mut arg_counter = 0;
        let mut result_counter = 0;

        for source_block_id in block_id_mapping.keys().copied() {
            let new_block_id = block_id_mapping[&source_block_id];

            let mut block_builder = builder.instr_seq(new_block_id);

            let (_, instructions) = self.blocks.get(&source_block_id).unwrap();

            for instruction in instructions.clone() {
                match instruction {
                    Instr::Block(inner) => {
                        let mapped_target = block_id_mapping[&inner.seq];
                        block_builder.instr(Instr::Block(Block { seq: mapped_target }));
                    },
                    Instr::Loop(inner) => {
                        let mapped_target = block_id_mapping[&inner.seq];
                        block_builder.instr(Instr::Loop(Loop { seq: mapped_target }));
                    },
                    Instr::Br(inner) => {
                        let mapped_target = block_id_mapping[&inner.block];
                        block_builder.instr(Instr::Br(Br {
                            block: mapped_target,
                        }));
                    },
                    Instr::BrIf(inner) => {
                        let mapped_target = block_id_mapping[&inner.block];
                        block_builder.instr(Instr::BrIf(BrIf {
                            block: mapped_target,
                        }));
                    },
                    Instr::BrTable(inner) => {
                        let blocks = inner
                            .blocks
                            .iter()
                            .map(|block| block_id_mapping[block])
                            .collect();
                        let default = block_id_mapping[&inner.default];
                        block_builder.instr(Instr::BrTable(BrTable { blocks, default }));
                    },

                    Instr::BrOnCast(inner) => {
                        let mapped_target = block_id_mapping[&inner.block];
                        block_builder.instr(Instr::BrOnCast(BrOnCast {
                            block: mapped_target,
                            ..inner.clone()
                        }));
                    },
                    Instr::BrOnCastFail(inner) => {
                        let mapped_target = block_id_mapping[&inner.block];
                        block_builder.instr(Instr::BrOnCastFail(BrOnCastFail {
                            block: mapped_target,
                            ..inner.clone()
                        }));
                    },
                    Instr::BrOnNull(inner) => {
                        let mapped_target = block_id_mapping[&inner.block];
                        block_builder.instr(Instr::BrOnNull(BrOnNull {
                            block: mapped_target,
                        }));
                    },
                    Instr::BrOnNonNull(inner) => {
                        let mapped_target = block_id_mapping[&inner.block];
                        block_builder.instr(Instr::BrOnNonNull(BrOnNonNull {
                            block: mapped_target,
                        }));
                    },

                    Instr::Call(inner) => {
                        if let Some(helper) = support_functions.get(&inner.func) {
                            match helper {
                                HelperFunctionDirection::Input(_) => {
                                    block_builder.local_get(arg_locals[arg_counter]);
                                    arg_counter += 1;
                                },
                                HelperFunctionDirection::Output(_) => {
                                    block_builder.local_set(results_locals[result_counter]);
                                    result_counter += 1;
                                },
                            }
                        } else {
                            block_builder.call(inner.func);
                        }
                    },

                    default => {
                        block_builder.instr(default);
                    },
                }
            }
        }

        let mut root_block = builder.func_body();
        for result in results_locals {
            root_block.local_get(result);
        }

        builder.finish(arg_locals, funcs)
    }
}
