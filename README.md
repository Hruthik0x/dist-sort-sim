## About
This program simulates distributed sorting on a line network using independent processes and socket-based communication. 
Each Node runs as a separate process and exchanges data with other Nodes through sockets.

It simulates : 
1) [Odd-Even Transposition](https://en.wikipedia.org/wiki/Odd%E2%80%93even_sort)
2) [Sasaki](https://www.sciencedirect.com/science/article/abs/pii/S0020019001003076)
3) [Triplet (Alternate n-1 round)](https://ieeexplore.ieee.org/document/5484861)

## Help Yourself
```
chmod +x simulate.sh
./simulate.sh --help
```

## Distributor Overview
The Distributor is responsible for managing and coordinating multiple Node processes to simulate a distributed sorting system over a line network. Its key responsibilities include:

- Launching Node Processes: The Distributor starts multiple Node instances, each as an independent process.
- Send order to Nodes :Assigning the number, algorithm and partial order.
- Defining Network Topology: The Distributor assigns each Node its neigbbouring Nodes to simulate a line network.
- Collecting Sorted Results: After sorting, the Distributor retrieves the final values from all Nodes.

## Node Overview
- Establishes a connection with the Distributor.
- Receives its assigned number, sorting algorithm, and partial order from the Distributor.
- Communicates with its neighbor Nodes to perform distributed sorting.
- Reports its final number after all the rounds back to the Distributor.

## Detailed Workflow : 

### Initializing the Socket Server:
- The Distributor starts a socket server (on `dist-port` which is dynamically assigned by the OS)
to manage communication with the Nodes.

### Launching Nodes as Independent Processes:
- The Distributor spawns multiple Node processes.
- Each Node is launched with the Distributor’s port number (`dist-port`) as an argument.

### Nodes Establish Connection with the Distributor:
- Each Node connects to the Distributor’s socket server using the provided `dist-port`
- Nodes establish this connection as part of their initialization.

### Nodes Start Their Own Servers:
- Each Node also starts its own socket server on `node-port`, which is dynamically assigned by the OS.
- This allows peer-to-peer communication between Nodes.

### Nodes Report Their Availability to the Distributor:
- Each Node sends a message to the Distributor, reporting its `node-port`.
- The Distributor collects all `node-port` mappings.

### Distributor Assigns Node Details:
Once all Nodes have connected, the Distributor :
- Assigns each Node its number.
- Specifies the sorting algorithm to use.
- Provides a partial ordering constraint.
- Shares the node-port numbers of its neigbbours, enabling inter-node communication.

### Nodes Establish Peer-to-Peer Connections:
- After receiving their neigbbour information, each Node establishes direct connections to its assigned neigbbour Nodes.

### Nodes Signal Readiness to the Distributor:
- Once all required connections are established, each Node sends a "Ready" message to the Distributor.

### Distributor Initiates Sorting:
- Once the Distributor has received "Ready" messages from all Nodes, it sends a "Start" command to all Nodes.


### Nodes Execute Sorting Algorithm:
- Upon receiving the "Start" signal, Nodes begin sorting using the assigned algorithm.
- They exchange data with their neigbbours over the socket connections to facilitate distributed sorting.

### Nodes Report Final Results:
- Once sorting is complete, each Node sends its final number back to the Distributor.

## Additional info : 
- Each Node runs a socket server to accept connections from its neighboring Nodes while also connecting to its neighbors' socket servers.
- This setup ensures that two connections are established between each pair of Nodes.
- As a server, incoming connections are treated as read streams, while as a client, outgoing connections function as write streams.
- This design simulates uni-directional channels between Nodes, with two separate connections enabling full bi-directional communication.

