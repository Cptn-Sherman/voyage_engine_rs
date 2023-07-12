
pub const CHUNK_SIZE_F32: f32 = 32.0;
pub const CHUNK_SIZE_I32: i32 = CHUNK_SIZE_F32 as i32;
pub const CHUNK_SIZE_F32_MIDPOINT: f32 = CHUNK_SIZE_F32 / 2.0;
pub const CHUNK_SIZE_I32_MIDPOINT: i32 = CHUNK_SIZE_I32 / 2;

/// Macro for emulating a ternary operator in Rust.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// let x = 42;
/// let y = ternary!(x > 0, "positive", "non-positive");
/// println!("y: {}", y); // Output: y: positive
/// ```
///
/// # Syntax
///
/// The `ternary` macro takes three arguments:
///
/// * `$condition` - An expression that evaluates to a boolean condition.
/// * `$true_expr` - An expression to be evaluated if the condition is true.
/// * `$false_expr` - An expression to be evaluated if the condition is false.
///
/// # Notes
///
/// The `ternary` macro expands into an `if` statement that evaluates the condition and returns the corresponding expression.
///
/// It is important to note that macros should be used judiciously, considering the readability and maintainability of the code.
macro_rules! ternary {
    ($condition:expr, $true_expr:expr, $false_expr:expr) => {
        if $condition {
            $true_expr
        } else {
            $false_expr
        }
    };
}


/// Macro for creating a double for loop with a function callback.
///
/// # Examples
///
/// Basic usage:
///
/// ```rust
/// let numbers = vec![1, 2, 3];
/// let characters = vec!['A', 'B', 'C'];
///
/// double_for_loop!(num in &numbers, ch in &characters, {
///     println!("Number: {}, Character: {}", num, ch);
/// });
/// ```
///
/// # Syntax
///
/// The `double_for_loop` macro takes three arguments:
///
/// * `$var1` - An identifier to represent the current element in the first iterator.
/// * `in $iter1` - The first iterator to iterate over.
/// * `$var2` - An identifier to represent the current element in the second iterator.
/// * `in $iter2` - The second iterator to iterate over.
/// * `$callback` - A block of code to execute for each combination of elements from the two iterators.
///
/// # Notes
///
/// The `double_for_loop` macro expands into a nested for loop that iterates over the given iterators and invokes the provided callback for each combination of values.
///
/// It is important to note that macros should be used judiciously, considering the readability and maintainability of the code.
macro_rules! double_for_loop {
    ($var1:ident in $iter1:expr, $var2:ident in $iter2:expr, $callback:expr) => {
        for $var1 in $iter1 {
            for $var2 in $iter2 {
                $callback
            }
        }
    };
}


use std::fmt::Write;
use std::cmp::PartialOrd;
use num_traits::Zero;

/// Formats a value as a string with optional decimal digits and support for negative space formatting.
///
/// # Arguments
///
/// * `value` - The value to format.
/// * `decimal_digits` - The number of decimal digits to include. Pass `Some(digits)` for a specific number of digits, or `None` for no decimal digits.
/// * `format_negative_space` - Determines whether negative values should be padded with a leading space.
///
/// # Returns
///
/// A formatted string representation of the value.
///
/// # Examples
///
/// Formatting a positive value with 2 decimal places:
///
/// ```rust
/// let formatted = format_value(3.14, Some(2), false);
/// assert_eq!(formatted, " 3.14");
/// ```
///
/// Formatting a negative value without any decimal places:
///
/// ```rust
/// let formatted = format_value(-42, None, true);
/// assert_eq!(formatted, "-42");
/// ```
pub fn format_value<T: std::fmt::Display + Zero + PartialOrd>(
    value: T,
    decimal_digits: Option<usize>,
    format_negative_space: bool,
) -> String {
    let mut buffer = String::new();
    let value_str = value.to_string();
    let num_digits = if value_str == "0" {
        1 // Account for the single digit zero
    } else {
        value_str.replace(".", "").len() // Calculate the number of digits
    };

    let width = if value >= T::zero() || !format_negative_space {
        num_digits + decimal_digits.unwrap_or(0) // Add one extra space for positive values and decimal digits
    } else {
        num_digits + 1 + decimal_digits.unwrap_or(0) // Add two extra spaces for negative values (including the negative sign) and decimal digits
    };

    if format_negative_space && value >= T::zero() {
        write!(
            &mut buffer,
            " {:>width$.decimal_width$}",
            value,
            width = width,
            decimal_width = decimal_digits.unwrap_or(0)
        )
    } else {
        write!(
            &mut buffer,
            "{:>width$.decimal_width$}",
            value,
            width = width,
            decimal_width = decimal_digits.unwrap_or(0)
        )
    }
    .expect("Failed to write to buffer");

    buffer
}


/// Converts a coordinate to a chunk coordinate.
///
/// Chunks are square regions in a 2D grid. This function takes a coordinate
/// and returns the corresponding chunk coordinate. The chunk coordinate
/// represents the index of the chunk that contains the given coordinate.
///
/// # Arguments
///
/// * `coord` - The coordinate value to convert.
///
/// # Returns
///
/// The chunk coordinate that corresponds to the given coordinate.
///
/// # Examples
///
/// ```
/// let coord = -15;
/// let chunk_coord = convert_to_chunk_coordinate(coord);
/// assert_eq!(chunk_coord, -1);
/// ```
pub fn convert_to_chunk_coordinate(coord: i32) -> i32 {
    if coord < 0 {
        (coord + 1) / (CHUNK_SIZE_F32 as i32) - 1
    } else {
        coord / CHUNK_SIZE_F32 as i32
    }
}