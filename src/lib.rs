mod check;
mod context;
mod env;
mod eval;
mod expand;
mod gc_foreign;
mod special_forms;
mod value;
mod read;
mod toplevel;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(3 / 2, 1);
    }
}
