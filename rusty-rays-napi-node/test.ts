import {
    getConfig,
    getDefaultConfig,
    logDebug,
    logError,
    logInfo,
    logTrace,
    logWarn,
    Model,
    setConfig,
    shutdownLogger,
    type Sphere,
    Tracer,
} from './dist/index.js';
import {v4 as uuidV4} from 'uuid'

const testArtifactDir = './test-artifacts';

async function main() {

    console.log('\n');
    console.log('*** Hello World! ***');
    console.log('*** This is a test of the Rusty Rays NAPI Node.js binding ***');
    console.log('\n');

    let config = getDefaultConfig();
    console.log('loaded config: ', config);
    console.log('setting log files dir to: ', `${testArtifactDir}`);
    config.logFilesDir = `${testArtifactDir}`;
    console.log('setting log level to debug');
    config.logLevel = "debug";
    await setConfig(config);
    console.log('adjusted config: ', getConfig());

    logError('This is a rusty rays error message');
    logWarn('This is a rusty rays warning message');
    logInfo('This is a rusty rays info message');
    logDebug('THIS DEBUG MESSAGE SHOULD NOT BE SEEN');
    logTrace('THIS TRACE MESSAGE SHOULD NOT BE SEEN');
    await new Promise((resolve) => setTimeout(resolve, 100))

    const model = await Model.fromFilePath('../sample-files/single-sphere-test-extended.ray');
    const spheres: Record<string, Sphere> = await model.allSpheres;
    console.log(`loaded ${Object.keys(spheres).length} spheres from model`);
    const newSphere: Sphere = {
        uuid: uuidV4(),
        surface: "txt002",
        radius: 0.166667,
        position: {
            x: 0.471405,
            y: -0.471405,
            z: 1.11022e-16
        }
    }
    console.log('adding second sphere to model:', newSphere);
    await model.upsertSphere(newSphere);
    console.log("second sphere successfully added");
    const tracer = await Tracer.create(model);
    await tracer.renderToFile(`${testArtifactDir}/jsRender.png`);

    shutdownLogger();
}

main();
