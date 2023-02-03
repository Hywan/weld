use std::fmt;

/// `Data` is a wrapper around `&[u8]`. The main purpose is to have a custom
/// `Debug` implementation that truncates the inner data.
#[repr(transparent)]
pub struct Data<'a> {
    pub(crate) inner: &'a [u8],
}

impl<'a> Data<'a> {
    pub(crate) fn new(inner: &'a [u8]) -> Self {
        Self { inner }
    }
}

impl<'a> fmt::Debug for Data<'a> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let len = self.inner.len();

        if len > 10 {
            formatter.write_fmt(format_args!("Data({:0<2x?} ... truncated)", &self.inner[..10]))
        } else {
            formatter.write_fmt(format_args!("Data({:0<2x?})", &self.inner[..len]))
        }
    }
}
