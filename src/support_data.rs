use std::collections::{BTreeMap, HashSet};
use walrus::{
    ExportId, ExportItem, FunctionId, FunctionKind, ImportId, LocalFunction, Module,
    ModuleFunctions, ValType,
    ir::{Instr, Value},
};

#[derive(Debug)]
pub struct SupportData {
    support_imports: HashSet<ImportId>,
    support_exports: HashSet<ExportId>,
    support_functions: HashSet<FunctionId>,

    pub input_functions: BTreeMap<FunctionId, ValType>,
    pub output_functions: BTreeMap<FunctionId, ValType>,

    signature_metadata: BTreeMap<String, BTreeMap<String, FunctionId>>,
}

impl SupportData {
    pub fn scan(module: &Module) -> Self {
        let mut this = Self {
            support_imports: Default::default(),
            support_exports: Default::default(),
            support_functions: Default::default(),

            input_functions: Default::default(),
            output_functions: Default::default(),

            signature_metadata: Default::default(),
        };

        this.scan_io(module);
        this.scan_signatures(module);

        this
    }

    fn scan_io(&mut self, module: &Module) {
        let support_imports: HashSet<ImportId> = module
            .imports
            .iter()
            .filter(|import| import.name.starts_with("wasm_calling_support"))
            .map(|import| import.id())
            .collect();

        for function in module.functions() {
            if let FunctionKind::Import(imported) = &function.kind {
                if support_imports.contains(&imported.import) {
                    let function_type = module.types.get(imported.ty).as_function().unwrap();

                    if let Some(ty) = function_type.params().first() {
                        self.output_functions.insert(function.id(), ty.clone());
                    } else if let Some(ty) = function_type.results().first() {
                        self.input_functions.insert(function.id(), ty.clone());
                    }

                    self.support_functions.insert(function.id());
                }
            }
        }

        self.support_imports.extend(support_imports);
    }

    fn scan_signatures(&mut self, module: &Module) {
        for export in module.exports.iter() {
            if export.name.starts_with("WRAPMETA_SIG") {
                let mut parts = export.name.splitn(2, "__");
                let key = parts.next().unwrap();
                let function_name = parts.next().unwrap();

                let function_id = match &export.item {
                    ExportItem::Function(f) => f,
                    _ => panic!("Metadata must be a function"),
                };

                self.support_exports.insert(export.id());
                self.support_functions.insert(function_id.clone());

                self.signature_metadata
                    .entry(function_name.to_string())
                    .or_default()
                    .insert(key.to_string(), function_id.clone());
            }
        }
    }

    pub fn remove_from_module(&self, module: &mut Module) {
        for import in self.support_imports.iter() {
            module.imports.delete(*import);
        }

        for export in self.support_exports.iter() {
            module.exports.delete(*export);
        }

        for function in self.support_functions.iter() {
            module.funcs.delete(*function);
        }
    }

    pub fn get_function_signature(
        &self,
        funcs: &ModuleFunctions,
        name: &str,
    ) -> Option<(Vec<ValType>, Vec<ValType>)> {
        let data = self.signature_metadata.get(name)?;

        let sig_function = data.get("WRAPMETA_SIG")?;
        let sig_arg_count = value_to_usize(read_function_as_value_by_id(
            funcs,
            data.get("WRAPMETA_SIG_ARG_COUNT")?.clone(),
        ));
        let sig_ret_count = value_to_usize(read_function_as_value_by_id(
            funcs,
            data.get("WRAPMETA_SIG_RETURN_COUNT")?.clone(),
        ));

        let sig_function = match &funcs.get(sig_function.clone()).kind {
            FunctionKind::Local(f) => f,
            _ => panic!("Function must be local"),
        };

        let mut input_types = Vec::with_capacity(sig_arg_count);
        let mut output_types = Vec::with_capacity(sig_ret_count);

        for (instr, _) in sig_function.block(sig_function.entry_block()).instrs.iter() {
            match instr {
                Instr::Call(call) => {
                    let ty = self
                        .input_functions
                        .get(&call.func)
                        .expect("Calls in signature must be to input functions")
                        .clone();
                    if input_types.len() < sig_arg_count {
                        input_types.push(ty);
                    } else {
                        output_types.push(ty);
                    }
                },
                _ => (),
            }
        }

        assert_eq!(input_types.len(), sig_arg_count);
        assert_eq!(output_types.len(), sig_ret_count);

        Some((input_types, output_types))
    }
}

fn read_function_as_value_by_id(funcs: &ModuleFunctions, id: FunctionId) -> Value {
    match &funcs.get(id).kind {
        FunctionKind::Local(f) => read_function_as_value(f),
        _ => panic!("Function must be local"),
    }
}
fn read_function_as_value(function: &LocalFunction) -> Value {
    match &function.block(function.entry_block()).instrs[0].0 {
        Instr::Const(value) => value.value.clone(),
        _ => panic!("Function is not a constant"),
    }
}

fn value_to_usize(value: Value) -> usize {
    match value {
        Value::I32(v) => v as usize,
        Value::I64(v) => v as usize,
        Value::F32(v) => v as usize,
        Value::F64(v) => v as usize,
        Value::V128(v) => v as usize,
    }
}
