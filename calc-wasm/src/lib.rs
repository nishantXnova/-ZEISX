use calc_lang::{calc, fmt_op};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn calculate(input: &str) -> String {
    let exp = input.trim();
    if exp.is_empty() { return String::from("Enter an expression"); }
    match calc::evaluate(exp) {
        Ok(n) => {
            if let Some(p) = calc::fast_two(exp) {
                format!("{} {} {} = {}", p.0, fmt_op(p.1), p.2, n)
            } else {
                format!("  => {}", n)
            }
        }
        Err(e) => format!("error: {}", e),
    }
}
