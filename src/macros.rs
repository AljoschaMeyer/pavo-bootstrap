use im_rc::OrdMap as ImOrdMap;

use crate::value::{Value, Id, Builtin};

pub fn default() -> ImOrdMap<Id, Value> {
    let mut m = ImOrdMap::new();

    env_add(&mut m, "quote", Builtin::MacroQuote);
    env_add(&mut m, "do", Builtin::MacroDo);
    env_add(&mut m, "set!", Builtin::MacroSetBang);
    env_add(&mut m, "throw", Builtin::MacroThrow);
    env_add(&mut m, "if", Builtin::MacroIf);
    env_add(&mut m, "let", Builtin::MacroLet);
    env_add(&mut m, "fn", Builtin::MacroFn);

    return m;
}

fn env_add(
    m: &mut ImOrdMap<Id, Value>,
    name: &str,
    b: Builtin,
) {
    m.insert(
        Id::user(name),
        Value::builtin(b),
    );
}