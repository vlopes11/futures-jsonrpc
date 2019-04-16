use crate::futures::prelude::*;
use crate::{ErrorVariant, JrpcRequest, JrpcResponse};

pub trait JrpcMethodTrait {
    fn generate_future<'a>(
        &self,
        request: JrpcRequest,
    ) -> Result<Box<'a + Future<Item = Option<JrpcResponse>, Error = ErrorVariant>>, ErrorVariant>;
}
