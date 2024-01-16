use crate::*;
use futures::future::BoxFuture;
use std::any::{Any, TypeId};
use ::type_sets::Members;

/// Automatically implemented when [`SendsProtocol`] is implemented for a protocol
/// that implements [`DynFromInto`].
pub trait DynSends: IsSender + Send + 'static {
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>>;

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>>;

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynTrySendError<BoxedMsg<Self::With>>>;

    fn members(&self) -> &'static [TypeId];

    fn clone_boxed(&self) -> BoxedSender<Self::With>;

    fn as_any(&self) -> &dyn Any;
}

impl<T> DynSends for T
where
    T: SendsProtocol + Clone + Send + Sync + 'static,
    T::Protocol: DynFromInto,
    T::With: Send,
{
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>> {
        Box::pin(async move {
            let (protocol, with) =
                T::Protocol::try_from_boxed_msg(msg).map_err(DynSendError::NotAccepted)?;

            T::send_protocol_with(self, protocol, with).await.map_err(
                |SendError((protocol, with))| DynSendError::Closed(protocol.into_boxed_msg(with)),
            )
        })
    }

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynSendError::NotAccepted)?;

        T::send_protocol_blocking_with(self, protocol, with).map_err(
            |SendError((protocol, with))| DynSendError::Closed(protocol.into_boxed_msg(with)),
        )
    }

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynTrySendError<BoxedMsg<Self::With>>> {
        let (protocol, with) =
            T::Protocol::try_from_boxed_msg(msg).map_err(DynTrySendError::NotAccepted)?;

        T::try_send_protocol_with(self, protocol, with).map_err(|e| match e {
            TrySendError::Closed((protocol, with)) => {
                DynTrySendError::Closed(protocol.into_boxed_msg(with))
            }
            TrySendError::Full((protocol, with)) => {
                DynTrySendError::Full(protocol.into_boxed_msg(with))
            }
        })
    }

    fn members(&self) -> &'static [TypeId] {
        <T::Protocol as Members>::members()
    }

    fn clone_boxed(&self) -> BoxedSender<Self::With> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl<W, T> From<T> for BoxedSender<W>
where
    T: SendsProtocol<With = W> + Clone + Send + Sync + 'static,
    T::Protocol: DynFromInto,
    W: Send + 'static,
{
    fn from(sender: T) -> Self {
        Box::new(sender)
    }
}

impl<W: 'static> DynSends for BoxedSender<W> {
    fn dyn_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> BoxFuture<Result<(), DynSendError<BoxedMsg<Self::With>>>> {
        (**self).dyn_send_boxed_msg_with(msg)
    }

    fn dyn_send_boxed_msg_blocking_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynSendError<BoxedMsg<Self::With>>> {
        (**self).dyn_send_boxed_msg_blocking_with(msg)
    }

    fn dyn_try_send_boxed_msg_with(
        &self,
        msg: BoxedMsg<Self::With>,
    ) -> Result<(), DynTrySendError<BoxedMsg<Self::With>>> {
        (**self).dyn_try_send_boxed_msg_with(msg)
    }

    fn members(&self) -> &'static [TypeId] {
        (**self).members()
    }

    fn clone_boxed(&self) -> BoxedSender<Self::With> {
        (**self).clone_boxed()
    }

    fn as_any(&self) -> &dyn Any {
        (**self).as_any()
    }
}

impl<W> IsSender for BoxedSender<W> {
    type With = W;

    fn is_closed(&self) -> bool {
        (**self).is_closed()
    }

    fn capacity(&self) -> Option<usize> {
        (**self).capacity()
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn receiver_count(&self) -> usize {
        (**self).receiver_count()
    }

    fn sender_count(&self) -> usize {
        (**self).sender_count()
    }
}

impl<T: 'static> Clone for BoxedSender<T> {
    fn clone(&self) -> Self {
        (**self).clone_boxed()
    }
}
