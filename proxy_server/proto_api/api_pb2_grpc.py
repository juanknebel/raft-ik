# Generated by the gRPC Python protocol compiler plugin. DO NOT EDIT!
"""Client and server classes corresponding to protobuf-defined services."""
import grpc
import warnings

import proto_api.api_pb2 as api__pb2

GRPC_GENERATED_VERSION = "1.64.0"
GRPC_VERSION = grpc.__version__
EXPECTED_ERROR_RELEASE = "1.65.0"
SCHEDULED_RELEASE_DATE = "June 25, 2024"
_version_not_supported = False

try:
    from grpc._utilities import first_version_is_lower

    _version_not_supported = first_version_is_lower(
        GRPC_VERSION, GRPC_GENERATED_VERSION
    )
except ImportError:
    _version_not_supported = True

if _version_not_supported:
    warnings.warn(
        f"The grpc package installed is at version {GRPC_VERSION},"
        + f" but the generated code in api_pb2_grpc.py depends on"
        + f" grpcio>={GRPC_GENERATED_VERSION}."
        + f" Please upgrade your grpc module to grpcio>={GRPC_GENERATED_VERSION}"
        + f" or downgrade your generated code using grpcio-tools<={GRPC_VERSION}."
        + f" This warning will become an error in {EXPECTED_ERROR_RELEASE},"
        + f" scheduled for release on {SCHEDULED_RELEASE_DATE}.",
        RuntimeWarning,
    )


class EntryPointStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.Command = channel.unary_unary(
            "/raft_api.EntryPoint/Command",
            request_serializer=api__pb2.CommandRequest.SerializeToString,
            response_deserializer=api__pb2.CommandResponse.FromString,
            _registered_method=True,
        )


class EntryPointServicer(object):
    """Missing associated documentation comment in .proto file."""

    def Command(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Method not implemented!")
        raise NotImplementedError("Method not implemented!")


def add_EntryPointServicer_to_server(servicer, server):
    rpc_method_handlers = {
        "Command": grpc.unary_unary_rpc_method_handler(
            servicer.Command,
            request_deserializer=api__pb2.CommandRequest.FromString,
            response_serializer=api__pb2.CommandResponse.SerializeToString,
        ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
        "raft_api.EntryPoint", rpc_method_handlers
    )
    server.add_generic_rpc_handlers((generic_handler,))
    server.add_registered_method_handlers("raft_api.EntryPoint", rpc_method_handlers)


# This class is part of an EXPERIMENTAL API.
class EntryPoint(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def Command(
        request,
        target,
        options=(),
        channel_credentials=None,
        call_credentials=None,
        insecure=False,
        compression=None,
        wait_for_ready=None,
        timeout=None,
        metadata=None,
    ):
        return grpc.experimental.unary_unary(
            request,
            target,
            "/raft_api.EntryPoint/Command",
            api__pb2.CommandRequest.SerializeToString,
            api__pb2.CommandResponse.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True,
        )


class HealthStub(object):
    """Missing associated documentation comment in .proto file."""

    def __init__(self, channel):
        """Constructor.

        Args:
            channel: A grpc.Channel.
        """
        self.Info = channel.unary_unary(
            "/raft_api.Health/Info",
            request_serializer=api__pb2.EmptyRequest.SerializeToString,
            response_deserializer=api__pb2.InfoResponse.FromString,
            _registered_method=True,
        )


class HealthServicer(object):
    """Missing associated documentation comment in .proto file."""

    def Info(self, request, context):
        """Missing associated documentation comment in .proto file."""
        context.set_code(grpc.StatusCode.UNIMPLEMENTED)
        context.set_details("Method not implemented!")
        raise NotImplementedError("Method not implemented!")


def add_HealthServicer_to_server(servicer, server):
    rpc_method_handlers = {
        "Info": grpc.unary_unary_rpc_method_handler(
            servicer.Info,
            request_deserializer=api__pb2.EmptyRequest.FromString,
            response_serializer=api__pb2.InfoResponse.SerializeToString,
        ),
    }
    generic_handler = grpc.method_handlers_generic_handler(
        "raft_api.Health", rpc_method_handlers
    )
    server.add_generic_rpc_handlers((generic_handler,))
    server.add_registered_method_handlers("raft_api.Health", rpc_method_handlers)


# This class is part of an EXPERIMENTAL API.
class Health(object):
    """Missing associated documentation comment in .proto file."""

    @staticmethod
    def Info(
        request,
        target,
        options=(),
        channel_credentials=None,
        call_credentials=None,
        insecure=False,
        compression=None,
        wait_for_ready=None,
        timeout=None,
        metadata=None,
    ):
        return grpc.experimental.unary_unary(
            request,
            target,
            "/raft_api.Health/Info",
            api__pb2.EmptyRequest.SerializeToString,
            api__pb2.InfoResponse.FromString,
            options,
            channel_credentials,
            insecure,
            call_credentials,
            compression,
            wait_for_ready,
            timeout,
            metadata,
            _registered_method=True,
        )