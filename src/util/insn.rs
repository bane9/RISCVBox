#[derive(Copy, Clone)]
pub struct EncodedInsn<T, const N: usize> {
    insn: [T; N],
    size: usize,
}

impl<T, const N: usize> EncodedInsn<T, N> {
    pub fn new() -> EncodedInsn<T, N>
    where
        T: Default + Copy,
    {
        EncodedInsn {
            insn: [T::default(); N],
            size: 0,
        }
    }

    pub fn push(&mut self, insn: T) {
        assert!(self.size < N);

        self.insn[self.size] = insn;
        self.size += 1;
    }

    pub fn as_ptr(&self) -> *const T {
        self.insn.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.insn.as_mut_ptr()
    }

    pub fn size(&self) -> usize {
        self.size
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a EncodedInsn<T, N> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.insn.iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut EncodedInsn<T, N> {
    type Item = &'a mut T;
    type IntoIter = std::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.insn.iter_mut()
    }
}
