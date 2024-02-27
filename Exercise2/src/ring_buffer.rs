pub struct RingBuffer<T> {
    length_: usize,
    read_index: usize,
    write_index: usize,
    buffer: Vec<T>,
}

impl<T: Copy + Default> RingBuffer<T> {
/// Initialize the RingBuffer with a given length.
/// # Example
/// ```
/// let rb = RingBuffer::new(10);
/// ```
    pub fn new(length: usize) -> Self {
        // Create a new RingBuffer with `length` slots and "default" values.
        // Hint: look into `vec!` and the `Default` trait.
        RingBuffer {
            length_: length,
            read_index: 0,
            write_index: 0,
            buffer: vec![T::default(); length],
        }
    }

    pub fn reset(&mut self) {
        // Clear internal buffer and reset indices.
        self.read_index = 0;
        self.write_index = 0;
        self.buffer.iter_mut().for_each(|x| *x = T::default());
    }

    // `put` and `peek` write/read without advancing the indices.
    pub fn put(&mut self, value: T) {
        self.buffer[self.write_index] = value;
    }

    pub fn peek(&self) -> T {
        self.buffer[self.read_index]
    }

    pub fn get(&self, offset: usize) -> T {
        let index = (self.read_index + offset) % self.length_;
        self.buffer[index]
    }

    // `push` and `pop` write/read and advance the indices.
    pub fn push(&mut self, value: T) {
        self.buffer[self.write_index] = value;
        self.write_index = (self.write_index + 1) % self.length_;

        // If the buffer is full (write catches up to read), advance read_index as well
        if self.write_index == self.read_index {
            self.read_index = (self.read_index + 1) % self.length_;
        }
    }

    pub fn pop(&mut self) -> T {
        let value = self.buffer[self.read_index];
        self.read_index = (self.read_index + 1) % self.length_;
        value
    }

    pub fn get_read_index(&self) -> usize {
        self.read_index
    }

    pub fn set_read_index(&mut self, index: usize) {
        self.read_index = index % self.length_;
    }

    pub fn get_write_index(&self) -> usize {
        self.write_index
    }

    pub fn set_write_index(&mut self, index: usize) {
        self.write_index = index % self.length_;
    }

    pub fn len(&self) -> usize {
        // Return number of values currently in the buffer.
        if self.write_index >= self.read_index {
            self.write_index - self.read_index
        } else {
            self.length_ - self.read_index + self.write_index
        }
    }

    pub fn capacity(&self) -> usize {
        // Return the length of the internal buffer.
        self.length_
    }
}
