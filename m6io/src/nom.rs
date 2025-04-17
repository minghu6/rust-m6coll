use nom::{
    AsBytes, AsChar, Compare, IResult, Input, Offset, Parser,
    character::satisfy,
    combinator::{complete, opt, recognize, success},
    multi::{many_m_n, many0, many1},
};


////////////////////////////////////////////////////////////////////////////////
//// Macros

/// `0..=9`
#[macro_export]
macro_rules! DIGIT {
    () => {
        '0'..='9'
    };
}

/// `'a'..='z' | 'A'..='Z'`
#[macro_export]
macro_rules! ALPHA {
    () => {
        'a'..='z' | 'A'..='Z'
    };
}

/// `DIGIT![] | 'A'..='F' | 'a'..='f'`
#[macro_export]
macro_rules! HEXDIG {
    () => {
        DIGIT![] | 'A'..='F' | 'a'..='f'
    };
}


////////////////////////////////////////////////////////////////////////////////
//// Functions

pub fn hexdig<I>(input: I) -> IResult<I, I>
where
    I: Input + Offset,
    I::Item: AsChar,
{
    recognize(satisfy(is_hexdig)).parse(input)
}

pub fn alpha<I>(input: I) -> IResult<I, I>
where
    I: Input + Offset,
    I::Item: AsChar,
{
    recognize(satisfy(is_alpha)).parse(input)
}

pub fn digit<I>(input: I) -> IResult<I, I>
where
    I: Input + Offset,
    I::Item: AsChar,
{
    recognize(satisfy(is_digit)).parse(input)
}

////////////////////////////////////////
//// Is Functions

pub fn is_hexdig(b: char) -> bool {
    matches!(b, HEXDIG![])
}

pub fn is_alpha(b: char) -> bool {
    matches!(b, ALPHA![])
}

pub fn is_digit(b: char) -> bool {
    matches!(b, DIGIT![])
}

////////////////////////////////////////
//// Common Functions

pub fn empty<I>(input: I) -> IResult<I, I>
where
    I: Input + Offset + AsBytes,
    I::Item: AsChar,
{
    recognize(success("")).parse(input)
}

pub fn on_guard_many0<I, F>(
    f: F,
) -> impl Parser<
    I,
    Output = Vec<<F as Parser<I>>::Output>,
    Error = <F as Parser<I>>::Error,
>
where
    I: Clone + Input,
    F: Parser<I>,
{
    many0(complete(f))
}

pub fn on_guard_many1<I, F>(
    f: F,
) -> impl Parser<
    I,
    Output = Vec<<F as Parser<I>>::Output>,
    Error = <F as Parser<I>>::Error,
>
where
    I: Clone + Input,
    F: Parser<I>,
{
    many1(complete(f))
}

pub fn on_guard_many_m_n<I, E, F>(
    min: usize,
    max: usize,
    parser: F,
) -> impl Parser<I, Output = Vec<<F as Parser<I>>::Output>, Error = E>
where
    I: Clone + Input,
    F: Parser<I, Error = E>,
    E: nom::error::ParseError<I>,
{
    many_m_n(min, max, complete(parser))
}

pub fn on_guard_opt<I: Clone, E: nom::error::ParseError<I>, F>(
    f: F,
) -> impl Parser<I, Output = Option<<F as Parser<I>>::Output>, Error = E>
where
    F: Parser<I, Error = E>,
{
    opt(complete(f))
}
