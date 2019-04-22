use crate::futures::prelude::*;
use crate::{ErrorVariant, JrpcRequest, JrpcResponse};

pub trait JrpcMethodTrait<'a> {
    fn generate_future(
        &self,
        request: JrpcRequest,
    ) -> Result<Box<'a + Future<Item = Option<JrpcResponse>, Error = ErrorVariant>>, ErrorVariant>;
}
