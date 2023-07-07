pub struct LoopIter<'a, T> {
    start: usize,
    index: Option<usize>,
    data: &'a Vec<T>,
}

impl<'a, T> LoopIter<'a, T> {
    fn new(start: usize, data: &'a Vec<T>) -> Self {
        Self { start, index: None, data }
    }
}

impl<'a, T> Iterator for LoopIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let index = match self.index {
            Some(index) if index == self.start => return None,
            Some(index) => index,
            _ => self.start,
        };
        let out = self.data.get(index);
        self.index = Some((index + 1) % self.data.len());
        out
    }
}

/// 循环迭代器，丛指定头开始，循环到末尾，再从头开始循环
pub trait LoopIterExt<T> {
    /// [start] 指定开始位置
    fn loop_iter(&self, start: usize) -> LoopIter<T>;
}

impl<T> LoopIterExt<T> for Vec<T> {
    fn loop_iter(&self, start: usize) -> LoopIter<T> {
        LoopIter::new(start, self)
    }
}

#[cfg(test)]
mod test {
    use super::LoopIterExt;

    #[test]
    fn iter() {
        let items = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut it = items.loop_iter(1);

        let out = it.next().unwrap();
        assert_eq!(out, &1);

        let out = it.next().unwrap();
        assert_eq!(out, &2);

        let mut it = items.loop_iter(4);
        let out = it.next().unwrap();
        assert_eq!(out, &4);

        let b: Vec<i32> = items.loop_iter(3).map(|n| n.clone()).collect();
        assert_eq!(b, vec![3, 4, 5, 6, 7, 8, 9, 0, 1, 2]);
    }
}
