use std::ops::{Add, Index};
use std::fmt;

pub struct SizedBuffer<const N: usize> {
    data: Vec<u8>,
}

impl<const N: usize> SizedBuffer<N> {
    pub fn new() -> Self {
        SizedBuffer {
            data: Vec::with_capacity(N),
        }
    }
    
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        if bytes.len() > N {
            return Err(format!(
                "Buffer overflow: attempted to store {} bytes in Bytes<{}> buffer",
                bytes.len(), N
            ));
        }
        Ok(SizedBuffer { data: bytes })
    }
    
    pub fn push(&mut self, byte: u8) -> Result<(), String> {
        if self.data.len() >= N {
            return Err(format!(
                "Buffer overflow: Bytes<{}> is full, cannot add more bytes",
                N
            ));
        }
        self.data.push(byte);
        Ok(())
    }
    
    pub fn extend(&mut self, bytes: &[u8]) -> Result<(), String> {
        if self.data.len() + bytes.len() > N {
            return Err(format!(
                "Buffer overflow: adding {} bytes to Bytes<{}> would exceed capacity (current: {})",
                bytes.len(), N, self.data.len()
            ));
        }
        self.data.extend_from_slice(bytes);
        Ok(())
    }
    
    pub fn len(&self) -> usize {
        self.data.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    
    pub fn capacity(&self) -> usize {
        N
    }
    
    pub fn remaining(&self) -> usize {
        N - self.data.len()
    }
    
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    pub fn to_vec(self) -> Vec<u8> {
        self.data
    }
    
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl<const N: usize> Add for SizedBuffer<N> {
    type Output = Result<SizedBuffer<N>, String>;
    
    fn add(self, rhs: SizedBuffer<N>) -> Self::Output {
        if self.data.len() + rhs.data.len() > N {
            return Err(format!(
                "Buffer overflow: combining buffers would exceed Bytes<{}> capacity",
                N
            ));
        }
        
        let mut result = self;
        result.data.extend(rhs.data);
        Ok(result)
    }
}

impl<const N: usize> Add<Vec<u8>> for SizedBuffer<N> {
    type Output = Result<SizedBuffer<N>, String>;
    
    fn add(self, rhs: Vec<u8>) -> Self::Output {
        if self.data.len() + rhs.len() > N {
            return Err(format!(
                "Buffer overflow: {} + {} bytes exceeds Bytes<{}> capacity",
                self.data.len(), rhs.len(), N
            ));
        }
        
        let mut result = self;
        result.data.extend(rhs);
        Ok(result)
    }
}

impl<const N: usize> Index<usize> for SizedBuffer<N> {
    type Output = u8;
    
    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const N: usize> fmt::Display for SizedBuffer<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Bytes<{}>({}/{}): 0x{}", 
            N, 
            self.data.len(), 
            N,
            hex::encode(&self.data)
        )
    }
}

impl<const N: usize> fmt::Debug for SizedBuffer<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SizedBuffer")
            .field("capacity", &N)
            .field("len", &self.data.len())
            .field("data", &format!("0x{}", hex::encode(&self.data)))
            .finish()
    }
}

pub fn pack64_sized(value: u64) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

pub fn pack32_sized(value: u32) -> Vec<u8> {
    value.to_le_bytes().to_vec()
}

pub fn validate_buffer_size<const N: usize>(data: &[u8]) -> Result<(), String> {
    if data.len() > N {
        Err(format!(
            "Compile-time size check failed: payload is {} bytes but Bytes<{}> only allows {} bytes",
            data.len(), N, N
        ))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_sized_buffer_creation() {
        let buf: SizedBuffer<100> = SizedBuffer::new();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.capacity(), 100);
    }
    
    #[test]
    fn test_buffer_overflow_prevention() {
        let mut buf: SizedBuffer<10> = SizedBuffer::new();
        buf.extend(&[1, 2, 3, 4, 5]).unwrap();
        
        let result = buf.extend(&[6, 7, 8, 9, 10, 11]);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_sized_buffer_addition() {
        let mut buf1: SizedBuffer<100> = SizedBuffer::new();
        buf1.extend(&[1, 2, 3]).unwrap();
        
        let mut buf2: SizedBuffer<100> = SizedBuffer::new();
        buf2.extend(&[4, 5, 6]).unwrap();
        
        let result = (buf1 + buf2).unwrap();
        assert_eq!(result.len(), 6);
    }
}
