#[derive(Debug)]
pub(crate) struct Node<D, const N: usize> {
    inner: Vec<D>,
    children: [Option<Box<Node<D, N>>>; 4],
}

impl<D, const N: usize> Default for Node<D, N> {
    fn default() -> Self {
        Self {
            inner: Vec::new(),
            children: [None, None, None, None],
        }
    }
}
