#[macro_export]
macro_rules! create_command {
    ($action:expr, $success_msg:expr, $failure_msg:expr, $error_msg:expr) => {{
        match $action {
            Ok(id) => {
                println!($success_msg, id = id);
                Ok(())
            }
            Err(error) => {
                eprintln!($failure_msg, error = error);
                Err($error_msg.into())
            }
        }
    }};
}

#[macro_export]
macro_rules! update_command {
    ($action:expr, $success_msg:expr, $failure_msg:expr, $not_found_msg:expr, $error_msg:expr) => {{
        match $action {
            Ok(rows_affected) => {
                if rows_affected == 0 {
                    eprintln!($not_found_msg);
                    Err($error_msg.into())
                } else {
                    println!($success_msg);
                    Ok(())
                }
            }
            Err(error) => {
                eprintln!($failure_msg, error = error);
                Err($error_msg.into())
            }
        }
    }};
}

#[macro_export]
macro_rules! delete_command {
    ($action:expr, $success_msg:expr, $failure_msg:expr, $not_found_msg:expr, $error_msg:expr) => {{
        match $action {
            Ok(rows_affected) => {
                if rows_affected == 0 {
                    eprintln!($not_found_msg);
                    Err($error_msg.into())
                } else {
                    println!($success_msg);
                    Ok(())
                }
            }
            Err(error) => {
                eprintln!($failure_msg, error = error);
                Err($error_msg.into())
            }
        }
    }};
}
