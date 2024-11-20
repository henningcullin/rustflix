/// Macro to handle create (insert) commands.
///
/// This macro simplifies the implementation of create operations by handling the success
/// and error logging as well as the return value logic.
///
/// # Arguments
/// - `$action`: The action to perform, e.g., `actions::add_directory(path)`.
/// - `$success_msg`: The log message to print on success, with placeholders for any dynamic data.
/// - `$failure_msg`: The log message to print when the action returns an error, with placeholders for dynamic data (e.g., `error`).
/// - `$error_msg`: The error message to return in case of failure (this will be wrapped in a `Result::Err`).
///
/// # Example usage:
/// ```rust
/// create_command!(
///     actions::add_directory(path),
///     "Directory added successfully with id: {id}",
///     "Error when inserting directory: {error}, path: {path}",
///     "Failed to add directory"
/// );
/// ```
///
/// This will log a success message when the directory is added, or an error message if
/// something goes wrong during the insertion, and it will return the appropriate `Result`.
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

/// Macro to handle update commands.
///
/// # Arguments
/// - `$action`: The action to perform, e.g., `actions::edit_directory(id, path)`.
/// - `$success_msg`: The log message to print on success, with placeholders for any dynamic data.
/// - `$failure_msg`: The log message to print when the action returns an error, with placeholders for dynamic data (e.g., `error`).
/// - `$not_found_msg`: The log message to print when no rows are affected (e.g., `No rows were updated`).
/// - `$error_msg`: The error message to return in case of failure (this will be wrapped in a `Result::Err`).
///
/// # Example usage:
/// ```rust
/// update_command!(
///     actions::edit_directory(id, path),
///     "Successfully edited directory with id: {id}",
///     "Error editing directory: {error}, id: {id}",
///     "No directory found with id: {id}",
///     "Failed to edit directory"
/// );
/// ```
///
/// This will log the success or failure of the update operation and return an appropriate result.
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

/// Macro to handle delete commands.
///
/// # Arguments
/// - `$action`: The action to perform, e.g., `actions::remove_directory(id)`.
/// - `$success_msg`: The log message to print on success, with placeholders for any dynamic data.
/// - `$failure_msg`: The log message to print when the action returns an error, with placeholders for dynamic data (e.g., `error`).
/// - `$not_found_msg`: The log message to print when no rows are affected (e.g., `No rows were deleted`).
/// - `$error_msg`: The error message to return in case of failure (this will be wrapped in a `Result::Err`).
///
/// # Example usage:
/// ```rust
/// delete_command!(
///     actions::remove_directory(id),
///     "Successfully deleted directory with id: {id}",
///     "Error deleting directory: {error}, id: {id}",
///     "No directory found with id: {id}",
///     "Failed to delete directory"
/// );
/// ```
///
/// This will log the success or failure of the delete operation and return an appropriate result.
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
