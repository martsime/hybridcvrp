importScripts("./build/wasm/hybridcvrp.js");

hybridcvrp("./build/wasm/hybridcvrp_bg.wasm").then((wasm) => {

  let solver = hybridcvrp.Solver.new();
  onmessage = function (msg) {
    if (msg.data.type === "LOAD_INSTANCE") {
      solver.clear();
      loadInstance(msg.data.value)
      postMessage({ type: "LOAD_COMPLETE" })
    } else if (msg.data.type === "ITERATE") {
      let result = solver.iterate();
      postMessage({ type: "ITERATION_COMPLETE", value: result });
    } else if (msg.data.type === "UPDATE_CONFIG") {
      updateConfig(msg.data.value);
      postMessage({ type: "CONFIG_UPDATED" });
    }
  };

  function loadInstance(instance) {
    if (instance.nodes.length > 0) {
      instance.nodes.forEach((node) => {
        solver.add_node(node.id, node.demand, node.x, node.y);
      });
      solver.add_capacity(instance.property.capacity);
      solver.load_problem();
    }
  }

  function updateConfig(config) {
    // Set timelimit to 1 day
    solver.update_time_limit(86400);

    // Update from config
    solver.update_initial_individuals(config.initialIndividuals);
    solver.update_min_population_size(config.minimumPopulationSize);
    solver.update_generation_size(config.generationSize);
    solver.update_local_search_granularity(config.localSearchGranularity);
    solver.update_number_of_elites(config.numberOfElites);
    solver.update_feasibility_proportion_target(config.feasibilityProportionTarget);
    solver.update_rr_gamma(config.RRGamma);
    solver.update_rr_start_temp(config.RRStartTemp);
    solver.update_elite_start_temp(config.eliteStartTemp);
    solver.update_elite_gamma(config.eliteGamma);
    solver.update_elite_education(config.eliteEducation);
  }
});
