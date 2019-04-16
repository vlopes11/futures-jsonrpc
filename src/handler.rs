use crate::futures::prelude::*;
use crate::{ErrorVariant, JrpcMethodTrait, JrpcRequest, JrpcResponse};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct JrpcHandler<'a> {
    hm_methods: Arc<RwLock<HashMap<String, Box<dyn JrpcMethodTrait + 'a>>>>,
}

impl<'a> JrpcHandler<'a> {
    pub fn new() -> Result<Self, ErrorVariant> {
        let hm_methods = Arc::new(RwLock::new(HashMap::new()));
        let handler = JrpcHandler { hm_methods };
        Ok(handler)
    }

    pub fn register_method<T: ToString, F: JrpcMethodTrait + 'a>(
        &self,
        signature: T,
        jrpc_method: F,
    ) -> Result<&Self, ErrorVariant> {
        let signature = signature.to_string();
        let jrpc_method = Box::new(jrpc_method);
        {
            self.hm_methods
                .try_write()
                .map_err(|_| ErrorVariant::RwLockPoisoned)
                .and_then(|mut hm| {
                    hm.insert(signature, jrpc_method);
                    Ok(())
                })?;
        }
        Ok(self)
    }

    pub fn handle_message<'m, T: ToString>(
        &self,
        message: T,
    ) -> Result<Box<'m + Future<Item = Option<JrpcResponse>, Error = ErrorVariant>>, ErrorVariant>
    {
        let message = message.to_string();
        let request = JrpcRequest::parse(message)?;

        let future = {
            self.hm_methods
                .try_read()
                .map_err(|_| ErrorVariant::RwLockPoisoned)
                .and_then(|hm| {
                    hm.get(request.get_method())
                        .map(|method| Ok(method))
                        .unwrap_or(Err(ErrorVariant::MethodSignatureNotFound))
                        .and_then(|method| method.generate_future(request))
                })?
        };

        Ok(future)
    }
}
