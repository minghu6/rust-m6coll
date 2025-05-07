pub use traits::*;


////////////////////////////////////////////////////////////////////////////////
//// Structures


////////////////////////////////////////////////////////////////////////////////
//// Implementations


////////////////////////////////////////////////////////////////////////////////
//// Functions

////////////////////////////////////////////////////////////////////////////////
//// Modules

pub mod combinator {
    use nom::{
        IResult, Input, Offset, Parser,
        combinator::{complete, opt, recognize, success},
        multi::{fold_many0, fold_many1, many_m_n, many0, many1},
    };

    ////////////////////////////////////////////////////////////////////////////////
    //// Traits
    // trait InputAndKindError<I>: ParseError<I> {
    //     fn input_and_kind(self) -> (I, ErrorKind);
    // }

    ////////////////////////////////////////////////////////////////////////////////
    //// Structures


    ////////////////////////////////////////////////////////////////////////////////
    //// Implementations


    ////////////////////////////////////////////////////////////////////////////////
    //// Functions

    ////////////////////////////////////////
    //// On-Guard Functions

    pub fn empty<I>(input: I) -> IResult<I, I>
    where
        I: Input + Offset,
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

    pub fn on_guard_fold_many0<I, E, F, G, H, R>(
        parser: F,
        init: H,
        g: G,
    ) -> impl Parser<I, Output = R, Error = E>
    where
        I: Clone + Input,
        F: Parser<I, Error = E>,
        G: FnMut(R, <F as Parser<I>>::Output) -> R,
        H: FnMut() -> R,
        E: nom::error::ParseError<I>,
    {
        fold_many0(complete(parser), init, g)
    }

    pub fn on_guard_fold_many1<I, E, F, G, H, R>(
        parser: F,
        init: H,
        g: G,
    ) -> impl Parser<I, Output = R, Error = E>
    where
        I: Clone + Input,
        F: Parser<I, Error = E>,
        G: FnMut(R, <F as Parser<I>>::Output) -> R,
        H: FnMut() -> R,
        E: nom::error::ParseError<I>,
    {
        fold_many1(complete(parser), init, g)
    }
}

mod traits {
    use nom::AsChar;

    ////////////////////////////////////////////////////////////////////////////////
    //// Traits

    pub trait AsByte: Copy {
        // Required methods
        fn as_byte(self) -> u8;
    }

    ////////////////////////////////////////////////////////////////////////////////
    //// Implementations

    impl<T: AsChar> AsByte for T {
        #[inline]
        fn as_byte(self) -> u8 {
            self.as_char() as u8
        }
    }
}

pub mod byte {
    use std::{
        ascii::Char::{CharacterTabulation, Space},
        num::NonZeroUsize,
        str::FromStr,
    };

    use nom::{
        AsBytes, Err, FindToken, IResult, Input, IsStreaming, Mode, Needed,
        Offset, Parser,
        combinator::{map_res, recognize},
        error::{Error, ErrorKind, ParseError},
    };

    use super::{
        AsByte,
        combinator::{on_guard_many0, on_guard_many1},
    };

    ////////////////////////////////////////////////////////////////////////////////
    //// Macros

    /// `0..=9`
    #[macro_export]
    macro_rules! DIGIT {
        () => {
            b'0'..=b'9'
        };
    }

    /// `'a'..='z' | 'A'..='Z'`
    #[macro_export]
    macro_rules! ALPHA {
        () => {
            b'a'..=b'z' | b'A'..=b'Z'
        };
    }

    /// `DIGIT![] | 'A'..='F' | 'a'..='f'`
    #[macro_export]
    macro_rules! HEXDIG {
        () => {
            DIGIT![] | b'A'..=b'F' | b'a'..=b'f'
        };
    }

    ///
    /// printable US-ASCII (include space)
    ///
    /// `32..=126`
    #[macro_export]
    macro_rules! VCHAR {
        () => {
            SP..=b'~'
        };
    }

    #[macro_export]
    macro_rules! WS {
        () => {
            SP | HTAB
        };
    }

    ////////////////////////////////////////////////////////////////////////////////
    //// Constants

    const SP: u8 = Space.to_u8();
    const HTAB: u8 = CharacterTabulation.to_u8();

    ////////////////////////////////////////////////////////////////////////////////
    //// Structures

    /// Parser implementation for [satisfy]
    pub struct Satisfy<F, ME> {
        predicate: F,
        make_error: ME,
    }

    ////////////////////////////////////////////////////////////////////////////////
    //// Implementations

    impl<I, E: ParseError<I>, F, ME> Parser<I> for Satisfy<F, ME>
    where
        I: Input,
        <I as Input>::Item: AsByte,
        F: Fn(u8) -> bool,
        ME: Fn(I) -> E,
    {
        type Output = I::Item;
        type Error = E;

        #[inline(always)]
        fn process<OM: nom::OutputMode>(
            &mut self,
            i: I,
        ) -> nom::PResult<OM, I, Self::Output, Self::Error> {
            match (i).iter_elements().next().map(|t| {
                let b = (self.predicate)(t.as_byte());
                (t, b)
            }) {
                None => {
                    if OM::Incomplete::is_streaming() {
                        Err(Err::Incomplete(Needed::Size(unsafe {
                            NonZeroUsize::new_unchecked(1usize)
                        })))
                    }
                    else {
                        Err(Err::Error(OM::Error::bind(|| {
                            (self.make_error)(i)
                        })))
                    }
                }
                Some((_, false)) => {
                    Err(Err::Error(OM::Error::bind(|| (self.make_error)(i))))
                }
                Some((t, true)) => {
                    Ok((i.take_from(1), OM::Output::bind(|| t)))
                }
            }
        }
    }

    ////////////////////////////////////////////////////////////////////////////////
    //// Functions

    pub fn safe_as_str_parse<I: AsBytes, F: FromStr>(
        input: I,
    ) -> Result<F, <F as FromStr>::Err> {
        unsafe { std::str::from_utf8_unchecked(input.as_bytes()) }.parse::<F>()
    }

    pub fn safe_as_str<'i, I: AsBytes + 'i>(input: I) -> &'i str {
        unsafe {
            std::str::from_raw_parts(
                input.as_bytes().as_ptr(),
                input.as_bytes().len(),
            )
        }
    }

    pub fn safe_to_string<I: AsBytes>(input: I) -> String {
        unsafe { std::str::from_utf8_unchecked(input.as_bytes()) }.to_owned()
    }

    pub fn safe_to_opt_string<I: AsBytes>(input: I) -> Option<String> {
        let s = safe_to_string(input);

        if s.is_empty() { None } else { Some(s) }
    }

    pub fn crlf<I>(input: I) -> IResult<I, I, Error<I>>
    where
        I: Input + Offset,
        <I as Input>::Item: AsByte,
    {
        recognize((byte(b'\r'), byte(b'\n')))
            .parse(input)
            .map_err(|err| {
                err.map(|err| Error {
                    input: err.input,
                    code: ErrorKind::CrLf,
                })
            })
    }

    pub fn digit0<I>(input: I) -> IResult<I, I, Error<I>>
    where
        I: Input + Offset,
        <I as Input>::Item: AsByte,
    {
        recognize(on_guard_many0(digit)).parse(input)
    }

    pub fn digit1<I>(input: I) -> IResult<I, I, Error<I>>
    where
        I: Input + Offset,
        <I as Input>::Item: AsByte,
    {
        recognize(on_guard_many1(digit)).parse(input)
    }

    pub fn digit1_as_u64<I>(input: I) -> IResult<I, u64, Error<I>>
    where
        I: Input + Offset + AsBytes,
        <I as Input>::Item: AsByte,
    {
        map_res(recognize(on_guard_many1(digit)), safe_as_str_parse)
            .parse(input)
    }

    pub fn alpha0<I>(input: I) -> IResult<I, I, Error<I>>
    where
        I: Input + Offset,
        <I as Input>::Item: AsByte,
    {
        recognize(on_guard_many0(alpha)).parse(input)
    }

    pub fn alpha1<I>(input: I) -> IResult<I, I, Error<I>>
    where
        I: Input + Offset,
        <I as Input>::Item: AsByte,
    {
        recognize(on_guard_many1(alpha)).parse(input)
    }

    pub fn hexdig0<I>(input: I) -> IResult<I, I, Error<I>>
    where
        I: Input + Offset,
        <I as Input>::Item: AsByte,
    {
        recognize(on_guard_many0(hexdig)).parse(input)
    }

    pub fn hexdig1<I>(input: I) -> IResult<I, I, Error<I>>
    where
        I: Input + Offset,
        <I as Input>::Item: AsByte,
    {
        recognize(on_guard_many1(hexdig)).parse(input)
    }

    /// satisfy one byte
    pub fn satisfy<F, I>(
        predicate: F,
    ) -> impl Parser<I, Output = I::Item, Error = Error<I>>
    where
        I: Input,
        I::Item: AsByte,
        F: Fn(u8) -> bool,
    {
        Satisfy {
            predicate,
            make_error: |i: I| Error::from_error_kind(i, ErrorKind::Satisfy),
        }
    }

    pub fn byte<I>(b: u8) -> impl Parser<I, Output = I::Item, Error = Error<I>>
    where
        I: Input,
        <I as Input>::Item: AsByte,
    {
        Satisfy {
            predicate: move |i| b == i,
            make_error: |i: I| Error::from_error_kind(i, ErrorKind::Satisfy),
        }
    }

    pub fn one_of<I, T, E: ParseError<I>>(
        list: T,
    ) -> impl Parser<I, Output = I::Item, Error = E>
    where
        I: Input,
        <I as Input>::Item: AsByte,
        T: FindToken<u8>,
    {
        Satisfy {
            predicate: move |c: u8| list.find_token(c),
            make_error: move |i| E::from_error_kind(i, ErrorKind::OneOf),
        }
    }

    pub fn hexdig<I>(input: I) -> IResult<I, I::Item>
    where
        I: Input,
        I::Item: AsByte,
    {
        satisfy(is_hexdig).parse(input)
    }

    pub fn alpha<I>(input: I) -> IResult<I, I::Item>
    where
        I: Input,
        I::Item: AsByte,
    {
        satisfy(is_alpha).parse(input)
    }

    pub fn digit<I>(input: I) -> IResult<I, I::Item>
    where
        I: Input,
        I::Item: AsByte,
    {
        satisfy(is_digit).parse(input)
    }

    pub fn sp<I>(input: I) -> IResult<I, I::Item>
    where
        I: Input,
        I::Item: AsByte,
    {
        satisfy(is_sp).parse(input)
    }

    pub fn htab<I>(input: I) -> IResult<I, I::Item>
    where
        I: Input,
        I::Item: AsByte,
    {
        satisfy(is_htab).parse(input)
    }

    ///
    /// ```abnf
    /// ws = SP / HTAB
    /// ```
    ///
    pub fn ws<I>(input: I) -> IResult<I, I::Item>
    where
        I: Input,
        I::Item: AsByte,
    {
        satisfy(is_ws).parse(input)
    }

    ////////////////////////////////////////
    //// Is Functions

    pub fn is_hexdig<T: AsByte>(b: T) -> bool {
        matches!(b.as_byte(), HEXDIG![])
    }

    pub fn is_alpha<T: AsByte>(b: T) -> bool {
        matches!(b.as_byte(), ALPHA![])
    }

    pub fn is_digit<T: AsByte>(b: T) -> bool {
        matches!(b.as_byte(), DIGIT![])
    }

    pub fn is_sp<T: AsByte>(b: T) -> bool {
        b.as_byte() == SP
    }

    pub fn is_vchar<T: AsByte>(b: T) -> bool {
        matches!(b.as_byte(), VCHAR![])
    }

    pub fn is_htab<T: AsByte>(b: T) -> bool {
        b.as_byte() == HTAB
    }

    ///
    /// SP or HTAB
    ///
    pub fn is_ws<T: AsByte>(b: T) -> bool {
        matches!(b.as_byte(), WS![])
    }


    #[cfg(test)]
    mod tests {

        use super::*;
        use crate::ByteStr;

        #[test]
        fn test_satisfy() {
            println!("{:?}", ws(ByteStr::new("")));
            println!("{:?}", ws(ByteStr::new("\t ")));

            println!("{}", ' ' as u8);
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_satisfy_byte() {}
}
