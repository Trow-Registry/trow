from diagrams import Diagram, Cluster
from diagrams.k8s.network import Ingress, Service 
from diagrams.k8s.compute import StatefulSet, Pod
from diagrams.k8s.storage import PV
from diagrams.k8s.infra import Node

with Diagram("Standard Kubernetes Install", show=False, direction="LR"):
    ing = Ingress("trow.io")
    svc =  Service("trow-svc") 
    ing >> svc
    pod = Pod("trow")
    StatefulSet("trow-set") - pod
    pod - PV("data-vol")

    svc >> pod

    with Cluster("Nodes"):
        workers = [Node("Node 1"),
                   Node("Node 2"),
                   Node("Node 3")]

    workers >> ing
