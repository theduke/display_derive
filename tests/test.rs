#[macro_use] extern crate display_derive;

#[derive(Display)]
struct UnitStruct;

#[test]
fn test_unit_struct() {
    assert_eq!("UnitStruct", UnitStruct.to_string());
}

#[derive(Display)]
#[display(fmt="custom")]
struct UnitStructCustom;

#[test]
fn test_unit_struct_custom() {
    assert_eq!("custom", UnitStructCustom.to_string());
}

#[derive(Display)]
#[display(fmt="custom {}/{}", _0, _1)]
struct TupleStructCustom(usize, usize);

#[test]
fn test_tuple_struct_custom() {
    assert_eq!("custom 0/1", TupleStructCustom(0, 1).to_string());
}

#[derive(Display)]
#[display(fmt="custom {}/{}", a, b)]
struct Struct{
    a: usize,
    b: usize,
}

#[test]
fn test_struct() {
    assert_eq!("custom 0/1", Struct{a: 0, b: 1}.to_string());
}

#[derive(Display)]
enum FullEnum{
    #[display(fmt="unit")]
    Unit,
    #[display(fmt="tuple {}", _0)]
    Tuple(i32),
    #[display(fmt="struct {}/{}", a, b)]
    Struct {
        a: i32,
        b: i32,
    },
}

#[test]
fn test_full_enum() {
    assert_eq!(FullEnum::Unit.to_string(), "unit");
    assert_eq!(FullEnum::Tuple(22).to_string(), "tuple 22");
    assert_eq!(FullEnum::Struct{a: 1, b: 2}.to_string(), "struct 1/2");
}
