use super::Buffer;
use crate::core::*;

///
/// A buffer containing per instance data.
/// To send this data to a shader, use the [Program::use_instance_attribute] method.
///
pub struct InstanceBuffer {
    buffer: Buffer,
}

impl InstanceBuffer {
    ///
    /// Creates a new empty instance buffer.
    ///
    pub fn new(context: &Context) -> Self {
        Self {
            buffer: Buffer::new(context),
        }
    }

    ///
    /// Creates a new instance buffer and fills it with the given data. The data should be in the same format as specified in the shader.
    /// As an example, if specified as `vec3` in the shader it needs to be specified as an array of `Vector3<T>` where `T` is a primitive type that implements [BufferDataType], for example can be f16 or f32.
    ///
    pub fn new_with_data<T: BufferDataType>(context: &Context, data: &[T]) -> Self {
        Self {
            buffer: Buffer::new_with_data(context, data),
        }
    }

    ///
    /// Fills the instance buffer with the given data. The data should be in the same format as specified in the shader.
    /// As an example, if specified as `vec3` in the shader it needs to be specified as an array of `Vector3<T>` where `T` is a primitive type that implements [BufferDataType], for example can be f16 or f32.
    ///
    pub fn fill<T: BufferDataType>(&mut self, data: &[T]) {
        self.buffer.fill(data)
    }

    ///
    /// The number of values in the buffer.
    ///
    pub fn count(&self) -> u32 {
        self.buffer.attribute_count() * self.buffer.data_size
    }

    ///
    /// The number of instance attributes in the buffer.
    ///
    pub fn instance_count(&self) -> u32 {
        self.buffer.attribute_count()
    }

    pub(in crate::core) fn bind(&self) {
        self.buffer.bind();
    }

    pub(in crate::core) fn data_type(&self) -> u32 {
        self.buffer.data_type
    }

    pub(in crate::core) fn data_size(&self) -> u32 {
        self.buffer.data_size
    }

    pub(in crate::core) fn normalized(&self) -> bool {
        self.buffer.normalized
    }
}