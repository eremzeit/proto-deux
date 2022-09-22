const fs = require("fs").promises;

function getGenePoolFitnessFilePath(experimentKey, genePoolId) {
  return `../data/experiments/${experimentKey}/gene_pools/gene_pool_${genePoolId}/fitness.csv`;
}
async function processMultiPoolExperimentFitness(experimentKey, genePoolId) {
  try {
    let path = getGenePoolFitnessFilePath(experimentKey, genePoolId);
    const data = await fs.readFile(path, { encoding: "utf8" });

    let row_cells = data
      .split("\n")
      .map((line) => line.split(",").map((cell) => Number.parseInt(cell))).filter(row => row[0] % 100 === 0);

      let outputPath = `./data/${experimentKey}_${genePoolId}.csv`;
      let outputData = '';

      row_cells.forEach((row) => {
        row.forEach(cell => {
            outputData = `${outputData}${cell},`
        });
        outputData = `${outputData}\n`
      });


    await fs.writeFile(outputPath, outputData); 
  } catch (err) {
    console.log(err);
  }
}


processMultiPoolExperimentFitness('compare_chem_configs', 0).then(() => console.log('done')).catch((err) => {console.log(err)})