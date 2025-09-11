import {Model, Tracer} from './index.js'

async function main() {
    const model = await Model.fromFilePath('../sample-files/jacks.ray');
    const tracer = new Tracer(model);
    await tracer.renderToFile('render/jsRender.png');
}

main();
