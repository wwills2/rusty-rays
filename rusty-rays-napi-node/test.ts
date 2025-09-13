import {logError, logInfo, logWarn, Model, shutdownLogger, Tracer} from './index.js';

async function main() {

    console.log('\n');
    console.log('*** Hello World! ***');
    console.log('*** This is a test of the Rusty Rays NAPI Node.js binding ***');
    console.log('\n');

    logError('This is a rusty rays error message');
    logWarn('This is a rusty rays warning message');
    logInfo('This is a rusty rays info message');

    const model = await Model.fromFilePath('../sample-files/jacks.ray');
    const tracer = new Tracer(model);
    await tracer.renderToFile('render/jsRender.png');

    shutdownLogger();
}

main();
