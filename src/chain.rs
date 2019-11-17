use crate::Chain;
use std::error::Error as StdError;

impl<'a> Iterator for Chain<'a> {
    type Item = &'a (dyn StdError + 'static);

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next?;
        self.next = next.source();
        Some(next)
    }
}
