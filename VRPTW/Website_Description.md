# Vehicle Routing with Time Windows
We have evolved our existing Capacitated Vehicle Routing challenge into a Vehicle Routing Problem with Time Windows (VRPTW) challenge. The VRPTW is an established extension of [The classic Vehicle Routing Problem (VRP)](https://en.wikipedia.org/wiki/Vehicle_routing_problem), distinguished by the introduction of time window constraints for each customer, adding a temporal dimension to the already intricate tasks of fleet sizing, route planning, and capacity management. These additional constraints make the VRPTW a better reflection of real-world logistical challenges and opens up a broader landscape for algorithmic innovation. The presence of time windows makes the problem computationally more challenging and encourages the exploration of novel algorithmic frameworks.

## (Optional) Main Differences Between Old and New Challenge
1. **Customer (Node) Position Assignment:** The customers are distributed across the grid in a mix of random and clustered assignments. This differs from the original challenge where customers were only randomly dispersed. This choice is to more accurately reflect real-world scenarios.
2. **Time Windows:** The main difference between this challenge and the previous one lies in the constraint of a set time window during which each customer must be served and a set time the vehicle must return to the depot before. Time windows for randomly dispersed customers are drawn uniformly from a set interval which ensures that the assigned time window is always feasible, meaning the vehicle can depart the depot, reach the customer, perform the required service, and return on time. For clustered customers, their time windows are assigned based on a weighted average of the time window of their 'seed' customer (the customer around which the cluster is focused) and a randomly generated time window for the customer. This is chosen to reflect real-world scenarios. Time windows widths are set to be randomly selected from a tight window range of [10,60] units. A fixed ratio of customers, determined by the density parameter, are then assigned zero ready times, resulting in wider time windows for a set proportion of the customers.

## Challenge Formulation
The Vehicle Routing Problem with Time Windows (VRPTW) involves determining a set of cost-effective routes for a fleet of identical vehicles operating from a single depot to serve a geographically dispersed set of customers. Each vehicle has a fixed capacity and each customer has a known demand for goods and a defined time window during which service must begin. If a vehicle arrives before this time window, it must wait; if it arrives after, service is considered infeasible. The primary objective is to minimise the total distance the fleet must travel to deliver goods to all customers and return to the depot, such that:

1. Each customer is visited by exactly one vehicle,
2. The total demand serviced by each vehicle does not exceed its capacity,
3. Each vehicle starts and ends its route at the depot,
4. Service at each customer commences within the customer’s defined time window,
5. The number of vehicles utilised is less than a set fleet size, and
6. Vehicles wait if they arrive early, and service durations are accounted for within the schedule.

## Example

The following is an example of the Vehicle Routing Problem with Time Windows with configurable difficulty. Two parameters can be adjusted in order to vary the difficulty of the challenge instance:

- Parameter 1: $num\textunderscore{ }nodes$ is the number of customers (plus 1 depot) which are distributed across a grid of 1000x1000 with the depot at the centre (500, 500).  
- Parameter 2: $better\textunderscore{ }than\textunderscore{ }baseline$ is the factor by which a solution must  be better than the baseline value [link TIG challenges for explanation of baseline value].

Demand of each customer is selected independently and uniformly at random from the range [1, 35]. Each customer is assigned a time window between which they must be serviced. Service duration is set to a fixed value of 10 time units per customer. The maximum capacity of each vehicle is set to 200.


Consider an example instance with `num_nodes=8` and `better_than_baseline=0.8` with the `baseline=3875`:

```
# A sample generated example
CUSTOMER
CUST NO.  XCOORD.   YCOORD.   DEMAND    READY TIME  DUE DATE   SERVICE TIME
       0       500       500         0            0      2318             0
       1        75       250        10            0       868            10
       2       940       582        11          825       884            10
       3       398       419        22            0       682            10
       4       424       690         6          256       273            10
       5       143       482        19          674       717            10
       6       187       292        27            0      1785            10
       7       382       204         3            0       832            10
       8       465       274        25         1386      1437            10
       
max_capacity = 200 # total demand for each route must not exceed this number
fleet_capacity = 4 # the total number of routes must not exceed this number
max_total_distance = baseline*better_than_baseline = 3100 # (better_than_baseline * baseline) routes must have total distance under this number to be a solution 
```

The depot is the first node (node 0) with demand 0. The vehicle capacity is set to 200 and the fleet capacity to 4. In this example, routes must have a total distance of 3100 or less to be a solution.

Now consider the following routes:

```
Route 1: [0, 6, 1, 7, 8, 0]
Route 2: [0, 4, 2, 0]
Route 3: [0, 3, 5, 0]
```

When evaluating these routes, each route has demand less than 200, the number of vehicles used, 3, is less than the fleet capacity, the time windows are not violated, and the total distance is shorter than 3100, thereby these routes are a solution:

* Route 1: 
    * Depot -> 6 -> 1 -> 7 -> 8 -> Depot
    * Demand = 27 + 10 + 3 + 25 = 65
    * Distance = 376 + 120 + 310 + 109 + 229 = 1144
* Route 2: 
    * Depot -> 4 -> 2 -> Depot
    * Demand = 6 + 11 = 17
    * Distance = 205 + 527 + 448 = 1180
* Route 3: 
    * Depot -> 3 -> 5 -> Depot
    * Demand = 22 + 19 = 41
    * Distance = 130 + 263 + 357 = 750
* Total Distance = 1144 + 1180 + 750 = 3074

## Our Challenge
In TIG, the baseline route is determined by using Solomon's I1 insertion heuristic that iteratively inserts customers into routes based on a cost function that balances distance and time constraints. The routes are built one by one until all customers are served. The goal is to produce a solution better than the baseline’s total distance by at least the specified factor (`better_than_baseline`), while ensuring all VRPTW constraints are satisfied. Please see the challenge code for a precise specification.
