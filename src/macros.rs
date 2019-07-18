use im_rc::OrdMap as ImOrdMap;

use crate::value::{Value, Id, Builtin};

pub fn default() -> ImOrdMap<Id, Value> {
    let mut m = ImOrdMap::new();

    env_add(&mut m, "do", Builtin::MacroDo);
    env_add(&mut m, "cond", Builtin::MacroCond);
    env_add(&mut m, "set!", Builtin::MacroSetBang);
    env_add(&mut m, "throw", Builtin::MacroThrow);
    env_add(&mut m, "if", Builtin::MacroIf);
    env_add(&mut m, "let", Builtin::MacroLet);
    env_add(&mut m, "letfn", Builtin::MacroLetFn);
    env_add(&mut m, "fn", Builtin::MacroFn);
    env_add(&mut m, "lambda", Builtin::MacroLambda);
    env_add(&mut m, "->", Builtin::MacroThreadFirst);
    env_add(&mut m, "->>", Builtin::MacroThreadLast);
    env_add(&mut m, "as->", Builtin::MacroThreadAs);
    env_add(&mut m, "or", Builtin::MacroOr);
    env_add(&mut m, "and", Builtin::MacroAnd);
    env_add(&mut m, "||", Builtin::MacroOr2);
    env_add(&mut m, "&&", Builtin::MacroAnd2);
    env_add(&mut m, "quasiquote", Builtin::MacroQuasiquote);
    env_add(&mut m, "while", Builtin::MacroWhile);
    env_add(&mut m, "match", Builtin::MacroMatch);
    env_add(&mut m, "case", Builtin::MacroCase);
    env_add(&mut m, "loop", Builtin::MacroLoop);
    env_add(&mut m, "try", Builtin::MacroTry);

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
