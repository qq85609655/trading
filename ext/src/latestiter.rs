pub struct LatestIter<'a, T> {
    start: usize,
    end: usize,
    data: &'a Vec<T>,
}

impl<'a, T> DoubleEndedIterator for LatestIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.end <= self.start {
            return None;
        }
        self.end -= 1;
        Some(&self.data[self.end])
    }
}

impl<T> LatestIter<'_, T> {
    fn new(data: &Vec<T>, start: usize, end: usize) -> LatestIter<T> {
        LatestIter { start, end, data }
    }
}

impl<'a, T> Iterator for LatestIter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.start >= self.end {
            return None;
        }
        let out = self.data.get(self.start);
        self.start += 1;
        out
    }
}

pub trait LatestIterExt<T> {
    fn latest_iter(&self, limit: usize, offset: usize) -> LatestIter<T>;
}

impl<T> LatestIterExt<T> for Vec<T> {
    fn latest_iter(&self, limit: usize, offset: usize) -> LatestIter<T> {
        let length = self.len();
        let limit = if limit == usize::MAX { length } else { limit };
        let (start, take) = if length > limit + offset {
            (length - limit - offset, limit)
        } else if length > offset {
            (0, length - offset)
        } else {
            (0, 0)
        };
        LatestIter::new(self, start, start + take)
    }
}

#[cfg(test)]
mod test {
    use super::LatestIterExt;

    #[test]
    fn latest() {
        let items: Vec<i32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let out: Vec<i32> = items.latest_iter(3, 0).map(|x| x.clone()).collect();
        assert_eq!(out, vec![7, 8, 9]);

        let out: Vec<i32> = items.latest_iter(3, 0).rev().map(|x| x.clone()).collect();
        assert_eq!(out, vec![9, 8, 7]);

        let out: Vec<i32> = items.latest_iter(3, 3).map(|x| x.clone()).collect();
        assert_eq!(out, vec![4, 5, 6]);

        let out: Vec<i32> = items.latest_iter(3, 8).map(|x| x.clone()).collect();
        assert_eq!(out, vec![0, 1]);

        let out: Vec<i32> = items.latest_iter(3, 8).rev().map(|x| x.clone()).collect();
        assert_eq!(out, vec![1, 0]);

        let out: Vec<i32> = items.latest_iter(3, 9).map(|x| x.clone()).collect();
        assert_eq!(out, vec![0]);

        let out: Vec<i32> = items.latest_iter(3, 10).map(|x| x.clone()).collect();
        assert!(out.is_empty());

        let out: Vec<i32> = items.latest_iter(3, 11).map(|x| x.clone()).collect();
        assert!(out.is_empty());

        let out: Vec<i32> = items.latest_iter(10, 5).map(|x| x.clone()).collect();
        assert_eq!(out, vec![0, 1, 2, 3, 4]);

        let out: Vec<i32> = items.latest_iter(10, 5).rev().map(|x| x.clone()).collect();
        assert_eq!(out, vec![4, 3, 2, 1, 0]);
    }

    #[test]
    fn latest_empty() {
        let items: Vec<i32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let out: Vec<i32> = items.latest_iter(usize::MAX, 9).map(|x| x.clone()).collect();
        dbg!(out);
    }
}
