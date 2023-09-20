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

    pub fn new_from_slice(insn: &[T]) -> EncodedInsn<T, N>
    where
        T: Default + Copy,
    {
        assert!(insn.len() <= N);

        let mut encoded_insn = EncodedInsn::new();

        for i in 0..insn.len() {
            encoded_insn.insn[i] = insn[i];
        }

        encoded_insn.size = insn.len();

        encoded_insn
    }

    pub fn push(&mut self, insn: T) {
        assert!(self.size < N);

        self.insn[self.size] = insn;
        self.size += 1;
    }

    pub fn push_slice(&mut self, insn: &[T])
    where
        T: Copy,
    {
        assert!(self.size + insn.len() <= N);

        for i in 0..insn.len() {
            self.insn[self.size + i] = insn[i];
        }

        self.size += insn.len();
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

impl<T, const N: usize> core::fmt::Debug for EncodedInsn<T, N>
where
    T: core::fmt::Debug + core::fmt::LowerHex + Copy,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "[")?;

        for i in 0..self.size {
            write!(f, "{:x}", self.insn[i])?;

            if i < self.size - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
    }
}

impl<T, const N: usize> core::fmt::Display for EncodedInsn<T, N>
where
    T: core::fmt::Debug + core::fmt::LowerHex + Copy,
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "[")?;

        for i in 0..self.size {
            write!(f, "{:x}", self.insn[i])?;

            if i < self.size - 1 {
                write!(f, ", ")?;
            }
        }

        write!(f, "]")
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
