use std::{
    error::Error,
    num::{IntErrorKind, ParseIntError},
};

#[test]
fn basic() -> Result<(), Box<dyn Error>> {
    let source = "42x100";

    let x: i32;
    let y: i32;

    crate::scanfmt!(source, "{}x{}", x, y);

    assert_eq!(42, x);
    assert_eq!(100, y);

    let source = "xX_1234567654321_Xx";
    let x: u64;

    crate::scanfmt!(source, "xX_{:o}_Xx", x);
    assert_eq!(x, 0o1234567654321);

    Ok(())
}

#[test]
fn int_after_string() -> Result<(), Box<dyn Error>> {
    let source = "The ultimate answer is: 42";

    let prompt: String;
    let answer: i32;
    crate::scanfmt!(source, "{}{}", prompt, answer);

    assert_eq!("The ultimate answer is: ", prompt);
    assert_eq!(42, answer);

    Ok(())
}

#[test]
fn string_after_int() {
    // string is greedy, so it eats everything, making int parsing fail.

    fn inner() -> Result<(), crate::ScanError> {
        let source = "42 is the answer";

        let _answer: i32;
        let _descriptor: String;
        crate::scanfmt!(source, "{}{}", _answer, _descriptor);
        Ok(())
    }

    match inner() {
        Err(crate::ScanError::Custom(boxed)) => {
            let pie = boxed.downcast::<ParseIntError>().expect("should be PIE");
            assert_eq!(IntErrorKind::Empty, *pie.kind());
        }
        _ => panic!(),
    }
}
