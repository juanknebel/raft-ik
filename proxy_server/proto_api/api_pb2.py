# -*- coding: utf-8 -*-
# Generated by the protocol buffer compiler.  DO NOT EDIT!
# source: api.proto
# Protobuf Python Version: 5.26.1
"""Generated protocol buffer code."""
from google.protobuf import descriptor as _descriptor
from google.protobuf import descriptor_pool as _descriptor_pool
from google.protobuf import symbol_database as _symbol_database
from google.protobuf.internal import builder as _builder
# @@protoc_insertion_point(imports)

_sym_db = _symbol_database.Default()




DESCRIPTOR = _descriptor_pool.Default().AddSerializedFile(b'\n\tapi.proto\x12\x08raft_api\"!\n\x0e\x43ommandRequest\x12\x0f\n\x07\x63ommand\x18\x01 \x01(\t\"/\n\x0f\x43ommandResponse\x12\x0e\n\x06leader\x18\x01 \x01(\r\x12\x0c\n\x04term\x18\x02 \x01(\x04\"\x0e\n\x0c\x45mptyRequest\"J\n\x0cInfoResponse\x12\n\n\x02id\x18\x01 \x01(\r\x12\x0c\n\x04term\x18\x02 \x01(\x04\x12\x0e\n\x06leader\x18\x03 \x01(\r\x12\x10\n\x08vote_for\x18\x04 \x01(\r2L\n\nEntryPoint\x12>\n\x07\x43ommand\x12\x18.raft_api.CommandRequest\x1a\x19.raft_api.CommandResponse2@\n\x06Health\x12\x36\n\x04Info\x12\x16.raft_api.EmptyRequest\x1a\x16.raft_api.InfoResponseb\x06proto3')

_globals = globals()
_builder.BuildMessageAndEnumDescriptors(DESCRIPTOR, _globals)
_builder.BuildTopDescriptorsAndMessages(DESCRIPTOR, 'api_pb2', _globals)
if not _descriptor._USE_C_DESCRIPTORS:
  DESCRIPTOR._loaded_options = None
  _globals['_COMMANDREQUEST']._serialized_start=23
  _globals['_COMMANDREQUEST']._serialized_end=56
  _globals['_COMMANDRESPONSE']._serialized_start=58
  _globals['_COMMANDRESPONSE']._serialized_end=105
  _globals['_EMPTYREQUEST']._serialized_start=107
  _globals['_EMPTYREQUEST']._serialized_end=121
  _globals['_INFORESPONSE']._serialized_start=123
  _globals['_INFORESPONSE']._serialized_end=197
  _globals['_ENTRYPOINT']._serialized_start=199
  _globals['_ENTRYPOINT']._serialized_end=275
  _globals['_HEALTH']._serialized_start=277
  _globals['_HEALTH']._serialized_end=341
# @@protoc_insertion_point(module_scope)