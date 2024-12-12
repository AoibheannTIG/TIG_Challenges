# Motivation Behind Update

At The Innovation Game, we recognise the importance of incentivising algorithm development in problems of real-world interest and applicability. As a result, we are evolving our existing Capacitated Vehicle Routing challenge into a Vehicle Routing Problem with Time Windows (VRPTW) challenge. The VRPTW is an established extension of [The classic Vehicle Routing Problem (VRP)](https://en.wikipedia.org/wiki/Vehicle_routing_problem), distinguished by the introduction of time window constraints for each customer, adding a temporal dimension to the already intricate tasks of fleet sizing, route planning, and capacity management. These additional constraints make the VRPTW a better reflection of real-world logistical challenges and open up a broader landscape for algorithmic innovation. The presence of time windows makes the problem computationally more challenging and encourages the exploration of novel algorithmic frameworks.

# Vehicle Routing Problem with Time Windows

The Vehicle Routing Problem with Time Windows (VRPTW) involves determining a set of cost-effective routes for a fleet of identical vehicles operating from a single depot to serve a geographically dispersed set of customers. Each vehicle has a fixed capacity, and each customer has a known demand for goods and a defined time window during which service must begin. If a vehicle arrives before this time window, it must wait; if it arrives after, service is considered infeasible. The primary objective is to minimise the total distance the fleet must travel to deliver goods to all customers and return to the depot, such that:

- Each customer is visited by exactly one vehicle.
- The total demand serviced by each vehicle does not exceed its capacity.
- Each vehicle starts and ends its route at the depot.
- Service at each customer commences within the customer’s defined time window.
- The number of vehicles utilised is less than a set fleet size.
- Vehicles wait if they arrive early, and service durations are accounted for within the schedule.

# Example

The following is an example of the Vehicle Routing Problem with Time Windows with configurable difficulty. Two parameters can be adjusted in order to vary the difficulty of the challenge instance:

- **Parameter 1:** `num_nodes` is the number of customers, \(N\), plus 1 depot, in a \(\frac{N}{2} \times \frac{N}{2}\) grid with the depot at the centre \((\frac{N}{4}, \frac{N}{4})\).
- **Parameter 2:** `better_than_baseline` is the factor by which a solution must be better than the baseline value. Defines a factor by which a solution’s total travel distance must be improved relative to a given baseline solution. [See TIG challenges for explanation of baseline value.]

Demand of each customer is selected independently and uniformly at random from the range [1, 50]. Each customer, \(i\), is assigned a time window, \([e_i, l_i]\), between which they must be serviced. Service duration, \(s_i\), is set to a fixed value of 10 time units per customer. The maximum capacity of each vehicle is \(Q = 200\).

Consider an example instance with \(N=8\), `num_nodes=9` and `better_than_baseline=0.95`. Let the baseline distance be 15.07. The instance generation returns a text file of the following format:
