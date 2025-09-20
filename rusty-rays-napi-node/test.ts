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
    Tracer
} from './index.js';

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

    const model = await Model.fromFilePath('../sample-files/jacks.ray');
    const tracer = new Tracer(model);
    await tracer.renderToFile(`${testArtifactDir}/jsRender.png`);

    shutdownLogger();
}

main();
