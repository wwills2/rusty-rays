/**
 * utility for managing the reference to the currently open model.
 * the model has an internal semaphore.
 */

import { Model } from 'rusty-rays-napi-node';
let loadedModel: Model | undefined = undefined;

function getModel() {
  return loadedModel;
}

function setModel(model: Model | undefined) {
  loadedModel = model;
}

export { getModel, setModel };
