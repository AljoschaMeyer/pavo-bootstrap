mod check;
mod context;
mod env;
mod eval;
mod expand;
mod gc_foreign;
mod object;
mod read;
mod toplevel;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
