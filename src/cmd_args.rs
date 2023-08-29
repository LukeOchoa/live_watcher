use crate::err_tools;
use crate::MagicError;
use std::env;

// Vec<String>
pub fn get_arg() -> Result<Option<String>, MagicError> {
    let mut args = env::args().collect::<Vec<String>>();
    println!("args!!!!!! <{:?}>", args);

    let len = args.len();
    let result = match len {
        1 => Ok(None),
        2 => Ok(Some(args.remove(1))),
        _ => {
            let err_msg = &format!("You can only pass 1 (one) argument!");
            return Err(err_tools::ErrorX::new_box(err_msg));
        }
    };

    result
}
