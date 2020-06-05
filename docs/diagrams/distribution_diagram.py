from diagrams import Diagram, Cluster, Edge
from diagrams.k8s.network import Ingress, Service 
from diagrams.k8s.compute import StatefulSet, Pod
from diagrams.k8s.storage import PV
from diagrams.k8s.infra import Node

with Diagram("Advanced Distribution", show=False, direction="LR",
        graph_attr={"splines":"spline"}):


    with Cluster ("Node 1"):
        be1 = Pod("trow-backend-1");
        fe = Pod("trow-frontend");
        fe >> be1

    with Cluster ("Node 2"):
        be2 = Pod("trow-backend-2");

    with Cluster ("Node 3"):
        be3 = Pod("trow-backend-3");

    with Cluster ("Node 4"):
        be4 = Pod("trow-backend-4");

    with Cluster ("Node 5"):
        be5 = Pod("trow-backend-5");

    be1 - be2
    be1 - be3
    be1 - be4
    be1 - be5
    be2 - be3
    be2 - be4
    be2 - be5
    be3 - be4
    be3 - be5
    be4 - be5

    ing = Ingress("trow.io")
    svc =  Service("trow-svc") 
    ing >> svc
    svc >> fe

