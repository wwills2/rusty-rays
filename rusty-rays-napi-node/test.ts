import * as rustyRays from './dist/index.js'

async function main() {
    const model = await rustyRays.bindings.Model.fromFilePath('../sample-files/jacks.ray');
    const tracer = new rustyRays.bindings.Tracer(model);
    await tracer.renderToFile('jsRender.png');
}

main();
