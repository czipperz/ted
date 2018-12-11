pub type BufferContentsError = ();
pub type BufferContentsResult<T> = Result<T, BufferContentsError>;

#[cfg(not(test))]
const MAX_LENGTH_CHARS: usize = 1024;
#[cfg(test)]
const MAX_LENGTH_CHARS: usize = 4;
#[cfg(not(test))]
const PREFERRED_LENGTH_CHARS: usize = 512;
#[cfg(test)]
const PREFERRED_LENGTH_CHARS: usize = 2;

#[derive(Debug, Clone)]
pub struct BufferContents {
    array: Vec<S>,
}

impl BufferContents {
    pub fn new() -> Self {
        BufferContents { array: Vec::new() }
    }
}

#[derive(Debug, Clone)]
struct S {
    str: String,
    len_chars: usize,
}

impl BufferContents {
    pub fn len(&self) -> usize {
        let mut l = 0;
        for s in &self.array {
            l += s.len_chars;
        }
        l
    }

    pub fn iter<'a>(&'a self) -> BufferContentsIterator<'a> {
        if self.array.is_empty() {
            BufferContentsIterator {
                buffer_contents: self,
                outer: 0,
                inner: None,
            }
        } else {
            BufferContentsIterator {
                buffer_contents: self,
                outer: 0,
                inner: Some(self.array[0].str.chars()),
            }
        }
    }

    pub fn get(&self, mut loc: usize) -> BufferContentsResult<char> {
        for s in &self.array {
            if loc < s.len_chars {
                return Ok(s.str.chars().nth(loc).unwrap());
            } else {
                loc -= s.len_chars;
            }
        }
        Err(())
    }

    pub fn substring(&self, mut begin: usize, mut end: usize) -> BufferContentsResult<String> {
        let mut res = String::new();
        for s in &self.array {
            if res.is_empty() {
                if begin < s.len_chars {
                    if end < s.len_chars {
                        return Ok(s.str[begin..end].to_string());
                    } else {
                        res.push_str(&s.str[begin..]);
                    }
                } else {
                    begin -= s.len_chars;
                }
                end -= s.len_chars;
            } else {
                if end < s.len_chars {
                    return Ok(res);
                } else {
                    res.push_str(&s.str);
                    end -= s.len_chars;
                }
            }
        }
        Ok(res)
    }

    pub fn insert(&mut self, loc: usize, c: char) -> BufferContentsResult<()> {
        self.insert_str(loc, &c.to_string())
    }
    pub fn insert_str(&mut self, mut loc: usize, str: &str) -> BufferContentsResult<()> {
        let str_len = str.chars().count();
        for i in 0..self.array.len() {
            if loc <= self.array[i].len_chars {
                if str_len + self.array[i].str.chars().count() <= MAX_LENGTH_CHARS {
                    let j = self.array[i]
                        .str
                        .char_indices()
                        .skip(loc)
                        .next()
                        .map(|(j, _)| j)
                        .unwrap_or(self.array[i].str.len());
                    self.array[i].str.insert_str(j, str);
                    self.array[i].len_chars += str_len;
                } else {
                    let s = self.array.remove(i).str;
                    let mut i = i;
                    let mut it = MiddleChain::new(s.chars(), str.chars(), loc);
                    loop {
                        let s = it.take_(PREFERRED_LENGTH_CHARS);
                        if s.len_chars == 0 {
                            break;
                        } else if s.len_chars < PREFERRED_LENGTH_CHARS {
                            self.array.insert(i, s);
                            break;
                        } else {
                            self.array.insert(i, s);
                            i += 1;
                        }
                    }
                }
                return Ok(());
            }
            loc -= self.array[i].len_chars;
        }
        if loc == 0 {
            let mut i = 0;
            loop {
                if str_len == i {
                    break;
                } else if str_len < i + PREFERRED_LENGTH_CHARS {
                    self.array.push(S {
                        str: str.chars().skip(i).collect(),
                        len_chars: str_len - i,
                    });
                    break;
                } else {
                    self.array.push(S {
                        str: str.chars().skip(i).take(PREFERRED_LENGTH_CHARS).collect(),
                        len_chars: PREFERRED_LENGTH_CHARS,
                    });
                    i += PREFERRED_LENGTH_CHARS;
                }
            }
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn delete(&mut self, mut loc: usize) -> BufferContentsResult<()> {
        let mut i_ = None;
        for i in 0..self.array.len() {
            let s = &mut self.array[i];
            if loc < s.len_chars {
                {
                    let loc_chars = s.str.char_indices().skip(loc).next().unwrap().0;
                    s.str.remove(loc_chars);
                }
                s.len_chars -= 1;
                if s.len_chars == 0 {
                    i_ = Some(i);
                }
                loc = 0;
                break;
            } else {
                loc -= s.len_chars;
            }
        }
        match i_ {
            Some(i) => {
                self.array.remove(i);
                return Ok(());
            }
            None => {
                if loc == 0 {
                    Ok(())
                } else {
                    Err(())
                }
            }
        }
    }
    pub fn delete_region(&mut self, begin: usize, mut end: usize) -> BufferContentsResult<()> {
        while begin != end {
            self.delete(begin)?;
            end -= 1;
        }
        Ok(())
    }
}

impl<'a> From<&'a str> for BufferContents {
    fn from(s: &str) -> Self {
        let mut b = BufferContents::new();
        b.insert_str(0, s).unwrap();
        b
    }
}

struct MiddleChain<I1, I2> {
    iter1: I1,
    iter2: I2,
    state: MiddleChainState,
}

enum MiddleChainState {
    Front(usize),
    Middle,
    Back,
}

impl<I1, I2> MiddleChain<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
    fn new(iter1: I1, iter2: I2, offset: usize) -> Self {
        if offset == 0 {
            Self {
                iter1,
                iter2,
                state: MiddleChainState::Middle,
            }
        } else {
            Self {
                iter1,
                iter2,
                state: MiddleChainState::Front(offset),
            }
        }
    }
}

impl<I1, I2> MiddleChain<I1, I2>
where
    I1: Iterator<Item = char>,
    I2: Iterator<Item = char>,
{
    fn take_(&mut self, len: usize) -> S {
        let mut s = String::with_capacity(len);
        for l in 0..len {
            match self.next() {
                Some(c) => {
                    s.push(c);
                }
                None => {
                    return S {
                        str: s,
                        len_chars: l,
                    }
                }
            }
        }
        S {
            str: s,
            len_chars: len,
        }
    }
}

impl<I1, I2> Iterator for MiddleChain<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
    type Item = I1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.state {
            MiddleChainState::Front(i) => {
                if i == 1 {
                    self.state = MiddleChainState::Middle;
                } else {
                    self.state = MiddleChainState::Front(i - 1);
                }
                self.iter1.next()
            }
            MiddleChainState::Middle => {
                let n = self.iter2.next();
                if n.is_none() {
                    self.state = MiddleChainState::Back;
                    self.iter1.next()
                } else {
                    n
                }
            }
            MiddleChainState::Back => self.iter1.next(),
        }
    }
}

pub struct BufferContentsIterator<'a> {
    buffer_contents: &'a BufferContents,
    outer: usize,
    inner: Option<std::str::Chars<'a>>,
}

impl<'a> Iterator for BufferContentsIterator<'a> {
    type Item = char;
    fn next(&mut self) -> Option<char> {
        match self.inner.take() {
            Some(mut inner) => match inner.next() {
                None => {
                    self.outer += 1;
                    if self.outer >= self.buffer_contents.array.len() {
                        self.inner = None;
                        None
                    } else {
                        self.inner = Some(self.buffer_contents.array[self.outer].str.chars());
                        self.next()
                    }
                }
                c => {
                    self.inner = Some(inner);
                    c
                }
            },
            None => None,
        }
    }
}

use std::fmt;
impl fmt::Display for BufferContents {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.substring(0, self.len()).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_into_null() {
        let mut buf = BufferContents::new();
        assert!(buf.get(0).is_err());
        buf.insert_str(0, "abc").unwrap();
        assert_eq!(buf.get(0).unwrap(), 'a');
        assert_eq!(buf.get(1).unwrap(), 'b');
        assert_eq!(buf.get(2).unwrap(), 'c');
        assert!(buf.get(3).is_err());
    }

    #[test]
    fn insert_beginning() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "bcdefgh").unwrap();
        println!("{:?}", buf);
        assert_eq!(buf.array.len(), 4);
        assert_eq!(buf.get(0).unwrap(), 'b');
        assert_eq!(buf.get(1).unwrap(), 'c');
        assert_eq!(buf.get(2).unwrap(), 'd');
        assert_eq!(buf.get(3).unwrap(), 'e');
        assert_eq!(buf.get(4).unwrap(), 'f');
        assert_eq!(buf.get(5).unwrap(), 'g');
        assert_eq!(buf.get(6).unwrap(), 'h');
        buf.insert_str(0, "a").unwrap();
        println!("{:?}", buf);
        assert_eq!(buf.array.len(), 4);
        assert_eq!(buf.get(0).unwrap(), 'a');
        assert_eq!(buf.get(1).unwrap(), 'b');
        assert_eq!(buf.get(2).unwrap(), 'c');
        assert_eq!(buf.get(3).unwrap(), 'd');
        assert_eq!(buf.get(4).unwrap(), 'e');
        assert_eq!(buf.get(5).unwrap(), 'f');
        assert_eq!(buf.get(6).unwrap(), 'g');
        assert_eq!(buf.get(7).unwrap(), 'h');
    }

    #[test]
    fn insert_into_string() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "ac").unwrap();
        buf.insert_str(2, "d").unwrap();
        buf.insert_str(1, "b").unwrap();
        assert_eq!(buf.array.len(), 1);
        assert_eq!(buf.get(0).unwrap(), 'a');
        assert_eq!(buf.get(1).unwrap(), 'b');
        assert_eq!(buf.get(2).unwrap(), 'c');
        assert_eq!(buf.get(3).unwrap(), 'd');
        assert!(buf.get(4).is_err());

        buf = BufferContents::new();
        buf.insert_str(0, "acd").unwrap();
        assert_eq!(buf.len(), 3);
        assert_eq!(buf.array.len(), 2);
        buf.insert_str(1, "b").unwrap();
        assert_eq!(buf.len(), 4);
        assert_eq!(buf.array.len(), 2);
    }

    #[test]
    fn insert_inbetween_string() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "abgh").unwrap();
        buf.insert_str(2, "cdef").unwrap();
        println!("{:?}", buf);
        assert_eq!(buf.len(), 8);
        assert_eq!(buf.array.len(), 4);
        assert_eq!(buf.array[0].str, "ab");
        assert_eq!(buf.array[1].str, "cd");
        assert_eq!(buf.array[2].str, "ef");
        assert_eq!(buf.array[3].str, "gh");
        assert_eq!(buf.get(0).unwrap(), 'a');
        assert_eq!(buf.get(1).unwrap(), 'b');
        assert_eq!(buf.get(2).unwrap(), 'c');
        assert_eq!(buf.get(3).unwrap(), 'd');
        assert_eq!(buf.get(4).unwrap(), 'e');
        assert_eq!(buf.get(5).unwrap(), 'f');
        assert_eq!(buf.get(6).unwrap(), 'g');
        assert_eq!(buf.get(7).unwrap(), 'h');
    }

    #[test]
    fn insert_end_string() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "abcd").unwrap();
        println!("{:?}", buf);
        buf.insert_str(4, "efgh").unwrap();
        println!("{:?}", buf);
        assert_eq!(buf.len(), 8);
        assert_eq!(buf.array.len(), 4);
        assert_eq!(buf.array[0].str, "ab");
        assert_eq!(buf.array[1].str, "cd");
        assert_eq!(buf.array[2].str, "ef");
        assert_eq!(buf.array[3].str, "gh");
        assert_eq!(buf.get(0).unwrap(), 'a');
        assert_eq!(buf.get(1).unwrap(), 'b');
        assert_eq!(buf.get(2).unwrap(), 'c');
        assert_eq!(buf.get(3).unwrap(), 'd');
        assert_eq!(buf.get(4).unwrap(), 'e');
        assert_eq!(buf.get(5).unwrap(), 'f');
        assert_eq!(buf.get(6).unwrap(), 'g');
        assert_eq!(buf.get(7).unwrap(), 'h');
        assert!(buf.get(8).is_err());
    }

    #[test]
    fn insert_middle_string() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "abcd").unwrap();
        println!("{:?}", buf);
        buf.insert_str(4, "fghi").unwrap();
        println!("{:?}", buf);
        buf.insert_str(4, "e").unwrap();
        println!("{:?}", buf);
        assert_eq!(buf.len(), 9);
        assert_eq!(buf.array.len(), 4);
        assert_eq!(buf.array[0].str, "ab");
        assert_eq!(buf.array[1].str, "cde");
        assert_eq!(buf.array[2].str, "fg");
        assert_eq!(buf.array[3].str, "hi");
        assert_eq!(buf.get(0).unwrap(), 'a');
        assert_eq!(buf.get(1).unwrap(), 'b');
        assert_eq!(buf.get(2).unwrap(), 'c');
        assert_eq!(buf.get(3).unwrap(), 'd');
        assert_eq!(buf.get(4).unwrap(), 'e');
        assert_eq!(buf.get(5).unwrap(), 'f');
        assert_eq!(buf.get(6).unwrap(), 'g');
        assert_eq!(buf.get(7).unwrap(), 'h');
        assert_eq!(buf.get(8).unwrap(), 'i');
        assert!(buf.get(9).is_err());
    }

    #[test]
    fn insert_handle_char_boundary() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "aβ").unwrap();
        assert_eq!(buf.array.len(), 1);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.array[0].str, "aβ");

        buf.insert_str(2, "c").unwrap();
        assert_eq!(format!("{}", buf), "aβc");
    }

    #[test]
    fn delete_char_boundary() {
        let mut buf = BufferContents::new();
        buf.insert(0, 'β').unwrap();
        buf.insert(1, 'χ').unwrap();
        assert_eq!(format!("{}", buf), "βχ");

        {
            let mut buf = buf.clone();
            buf.delete(0).unwrap();
            assert_eq!(format!("{}", buf), "χ");
        }

        buf.insert(2, 'c').unwrap();
        assert_eq!(format!("{}", buf), "βχc");

        buf.delete(1).unwrap();
        assert_eq!(format!("{}", buf), "βc");
    }

    #[test]
    fn delete_beginning() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "ab").unwrap();
        buf.delete(0).unwrap();
        assert_eq!(buf.array.len(), 1);
        assert_eq!(buf.len(), 1);
        assert_eq!(buf.array[0].str, "b");
        assert_eq!(buf.array[0].len_chars, 1);

        buf.delete(0).unwrap();
        assert_eq!(buf.array.len(), 0);
        assert_eq!(buf.len(), 0);
    }

    #[test]
    fn delete_middle() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "ab").unwrap();
        buf.insert_str(2, "c").unwrap();
        assert_eq!(buf.array.len(), 1);
        buf.delete(1).unwrap();
        assert_eq!(buf.array.len(), 1);
        assert_eq!(buf.array[0].str, "ac");
        assert_eq!(buf.array[0].len_chars, 2);
    }

    #[test]
    fn delete_region_inside() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "ab").unwrap();
        buf.insert_str(2, "cd").unwrap();
        buf.delete_region(1, 3).unwrap();
        assert_eq!(buf.array.len(), 1);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.array[0].str, "ad");
        assert_eq!(buf.array[0].len_chars, 2);
    }

    #[test]
    fn delete_region_between() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "abcd").unwrap();
        assert_eq!(buf.array.len(), 2);
        buf.delete_region(1, 3).unwrap();
        assert_eq!(buf.array.len(), 2);
        assert_eq!(buf.len(), 2);
        assert_eq!(buf.array[0].str, "a");
        assert_eq!(buf.array[0].len_chars, 1);
        assert_eq!(buf.array[1].str, "d");
        assert_eq!(buf.array[1].len_chars, 1);
    }

    #[test]
    fn iter_empty() {
        let buf = BufferContents::new();
        assert!(buf.iter().next().is_none());
    }

    #[test]
    fn iter_one() {
        let mut buf = BufferContents::new();
        buf.insert(0, 'a').unwrap();
        let mut iter = buf.iter();
        assert_eq!(iter.next().unwrap(), 'a');
        assert!(iter.next().is_none());

        let mut iter = buf.iter();
        assert_eq!(iter.next().unwrap(), 'a');
        assert!(iter.next().is_none());
    }

    #[test]
    fn iter_multiple() {
        let mut buf = BufferContents::new();
        buf.insert_str(0, "abcd").unwrap();
        let mut iter = buf.iter();
        assert_eq!(iter.next().unwrap(), 'a');
        assert_eq!(iter.next().unwrap(), 'b');
        assert_eq!(iter.next().unwrap(), 'c');
        assert_eq!(iter.next().unwrap(), 'd');
        assert!(iter.next().is_none());
    }
}
