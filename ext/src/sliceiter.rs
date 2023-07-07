pub struct SliceIter<'a, T> {
    size: usize,
    index: usize,
    data: &'a Vec<T>,
}

impl<T> SliceIter<'_, T> {
    pub fn new(data: &Vec<T>, size: usize) -> SliceIter<T> {
        SliceIter { size, index: 0, data }
    }
}

impl<'a, T> Iterator for SliceIter<'a, T> {
    type Item = Vec<&'a T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.size == 0 {
            return None;
        }
        let mut out = vec![];
        if self.index >= self.data.len() {
            return None;
        }
        self.data.iter().skip(self.index).take(self.size).for_each(|n| out.push(n));

        self.index += self.size;
        Some(out)
    }
}

/// 这是一个扩展trait，用于给Vec增加slice_iter方法，这个方法返回一个SliceIter迭代器
/// SliceIter迭代器的next方法返回一个Vec<&T>，这个Vec的长度为size，当迭代到最后一个元素时，Vec的长度可能小于size
pub trait SliceIterExt<T> {
    fn slice_iter(&self, num: usize) -> SliceIter<T>;
}

impl<T> SliceIterExt<T> for Vec<T> {
    fn slice_iter(&self, num: usize) -> SliceIter<T> {
        SliceIter::new(self, num)
    }
}

#[cfg(test)]
mod test {
    use super::SliceIterExt;

    #[derive(Debug)]
    struct Num {
        n: i32,
    }

    impl Drop for Num {
        fn drop(&mut self) {
            println!("drop {}", self.n);
        }
    }

    #[test]
    fn test() {
        let v: Vec<Num> = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10].iter().map(|&n| Num { n }).collect();
        let mut v = v.slice_iter(3);
        for _ in 0..3 {
            let items = v.next();
            assert!(items.is_some());
            assert_eq!(3, items.unwrap().len());
        }

        let items = v.next();
        assert!(items.is_some());
        assert_eq!(1, items.unwrap().len());
    }
}
