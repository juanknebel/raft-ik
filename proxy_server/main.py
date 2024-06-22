from dotenv import load_dotenv
import os
from bottle import route, run, request, HTTPResponse
import proto_api.api_pb2_grpc as pb2_grpc
import proto_api.api_pb2 as pb2
import grpc


class ProxyConfig:
    def __init__(self) -> None:
        load_dotenv()
        node_ids = os.getenv("NODES").split(",")  # pyright: ignore
        self.the_nodes = {}
        for a_node_id in node_ids:
            node_key = "NODE_" + a_node_id
            self.the_nodes[int(a_node_id)] = os.getenv(node_key)
        self.the_host, the_port = os.getenv("HOST").split(":")  # pyright: ignore
        self.the_port = int(the_port)

    def host(self) -> str:
        return self.the_host

    def port(self) -> int:
        return self.the_port

    def nodes(self):
        return self.the_nodes


class RaftIK:
    def __init__(self, config: ProxyConfig) -> None:
        self.nodes = config.nodes()
        self.leader = -1
        self.term = -1
        self.update_leader()

    def send_request(self, command: str) -> HTTPResponse:
        with grpc.insecure_channel(self.nodes[self.leader]) as channel:
            stub = pb2_grpc.EntryPointStub(channel)
            request = pb2.CommandRequest(command=command)
            try:
                response: pb2.CommandResponse = stub.Command(request)
            except grpc.RpcError as e:
                code = e.code()
                details = e.details()
                if code == grpc.StatusCode.PERMISSION_DENIED:
                    self.update_leader()

                print(code)
                print(details)
                return HTTPResponse(status=403, body=details)

            else:
                print(response)
                return HTTPResponse(status=201)

    def update_leader(self) -> None:
        for v, k in self.nodes.items():
            with grpc.insecure_channel(k) as channel:
                stub = pb2_grpc.HealthStub(channel)
                request = pb2.EmptyRequest()
                response: pb2.InfoResponse = stub.Info(request)
                if response.term > self.term:
                    print(v)
                    self.leader = response.leader
                    self.term = response.term


proxy_config = ProxyConfig()
raft_ik = RaftIK(proxy_config)


@route("/ping", method="GET")
def ping():
    return "pong"


@route("/api/execute", method="POST")
def execute():
    body_json = request.json
    return raft_ik.send_request(body_json["command"])  # pyright: ignore


if __name__ == "__main__":
    run(host=proxy_config.host(), reloader=True, port=proxy_config.port())
