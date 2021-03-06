use crate::*;

#[derive(Debug, Clone)]
pub struct EnumInfo {
    method: IdentId,
    receiver: Value,
    args: Args,
}

impl EnumInfo {
    pub fn new(method: IdentId, receiver: Value, mut args: Args) -> Self {
        args.block = Some(MethodRef::from(0));
        EnumInfo {
            method,
            receiver,
            args,
        }
    }
}

pub type EnumRef = Ref<EnumInfo>;

impl EnumRef {
    pub fn from(method: IdentId, receiver: Value, args: Args) -> Self {
        EnumRef::new(EnumInfo::new(method, receiver, args))
    }

    pub fn eval(&self, vm: &mut VM) -> VMResult {
        let receiver = self.receiver;
        let method = vm.get_method(receiver, self.method)?;
        vm.eval_send(method, receiver, &self.args)
    }
}

pub fn init_enumerator(globals: &mut Globals) -> Value {
    let id = globals.get_ident_id("Enumerator");
    let class = ClassRef::from(id, globals.builtins.object);
    globals.add_builtin_instance_method(class, "each", each);
    globals.add_builtin_instance_method(class, "map", map);
    globals.add_builtin_instance_method(class, "collect", map);
    globals.add_builtin_instance_method(class, "with_index", with_index);
    globals.add_builtin_instance_method(class, "inspect", inspect);
    let class = Value::class(globals, class);
    globals.add_builtin_class_method(class, "new", enum_new);
    class
}

// Class methods

fn enum_new(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_min(args.len(), 1)?;
    let (receiver, method, new_args) = if args.len() == 1 {
        let method = vm.globals.get_ident_id("each");
        let new_args = Args::new0();
        (self_val, method, new_args)
    } else {
        if !args[1].is_packed_symbol() {
            return Err(vm.error_argument("2nd arg must be Symbol."));
        };
        let method = args[1].as_packed_symbol();
        let mut new_args = Args::new(args.len() - 2);
        for i in 0..args.len() - 2 {
            new_args[i] = args[i + 2];
        }
        new_args.block = None;
        (args[0], method, new_args)
    };
    let val = Value::enumerator(&vm.globals, method, receiver, new_args);
    Ok(val)
}

// Instance methods

fn inspect(vm: &mut VM, self_val: Value, _args: &Args) -> VMResult {
    let eref = vm.expect_enumerator(self_val, "Expect Enumerator.")?;
    let arg_string = {
        match eref.args.len() {
            0 => "".to_string(),
            1 => vm.val_inspect(eref.args[0]),
            _ => {
                let mut s = vm.val_inspect(eref.args[0]);
                for i in 1..eref.args.len() {
                    s = format!("{},{}", s, vm.val_inspect(eref.args[i]));
                }
                s
            }
        }
    };
    let inspect = format!(
        "#<Enumerator: {}:{}({})>",
        vm.val_inspect(eref.receiver),
        vm.globals.get_ident_name(eref.method),
        arg_string
    );
    Ok(Value::string(&vm.globals, inspect))
}

fn each(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0)?;
    let eref = vm.expect_enumerator(self_val, "Expect Enumerator.")?;
    let block = match args.block {
        Some(method) => method,
        None => {
            return Ok(self_val);
        }
    };

    let val = vm.eval_enumerator(eref)?;

    let ary = vm.expect_array(val, "Base object")?;
    let mut args = Args::new1(Value::nil());
    for elem in &ary.elements {
        args[0] = *elem;
        let _ = vm.eval_block(block, &args)?;
    }
    Ok(val)
}

fn map(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0)?;
    let eref = vm.expect_enumerator(self_val, "Expect Enumerator.")?;
    let block = match args.block {
        Some(method) => method,
        None => {
            // return Enumerator
            let id = vm.globals.get_ident_id("map");
            let e = Value::enumerator(&vm.globals, id, self_val, args.clone());
            return Ok(e);
        }
    };
    let val = vm.eval_enumerator(eref)?;

    let ary = vm.expect_array(val, "Base object")?;
    let mut args = Args::new1(Value::nil());
    let mut res = vec![];
    for elem in &ary.elements {
        args[0] = *elem;
        res.push(vm.eval_block(block, &args)?);
    }
    Ok(Value::array_from(&vm.globals, res))
}

fn with_index(vm: &mut VM, self_val: Value, args: &Args) -> VMResult {
    vm.check_args_num(args.len(), 0)?;
    let eref = vm.expect_enumerator(self_val, "Expect Enumerator.")?;
    let block = match args.block {
        Some(method) => method,
        None => {
            // return Enumerator
            let id = vm.globals.get_ident_id("with_index");
            let e = Value::enumerator(&vm.globals, id, self_val, args.clone());
            return Ok(e);
        }
    };

    let val = vm.eval_enumerator(eref)?;
    let res_ary: Vec<(Value, Value)> = vm
        .expect_array(val, "Base object")?
        .elements
        .iter()
        .enumerate()
        .map(|(i, v)| (v.clone(), Value::fixnum(i as i64)))
        .collect();

    let mut res = vec![];
    let mut arg = Args::new(2);

    for (v, i) in &res_ary {
        arg[0] = *v;
        arg[1] = *i;
        let val = vm.eval_block(block, &arg)?;
        res.push(val);
    }

    let res = Value::array_from(&vm.globals, res);
    Ok(res)
}

#[cfg(test)]
mod test {
    use crate::test::*;

    #[test]
    fn enumerator_with_index() {
        let program = r#"
        ans = %w(This is a Ruby.).map.with_index {|x| x }
        assert ["This", "is", "a", "Ruby."], ans
        ans = %w(This is a Ruby.).map.with_index {|x,y| [x,y] }
        assert [["This", 0], ["is", 1], ["a", 2], ["Ruby.", 3]], ans
        ans = %w(This is a Ruby.).map.with_index {|x,y,z| [x,y,z] }
        assert [["This", 0, nil], ["is", 1, nil], ["a", 2, nil], ["Ruby.", 3, nil]], ans
        "#;
        assert_script(program);
    }
}
