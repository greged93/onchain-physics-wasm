import('./node_modules/onchain-physics/onchain_physics_bg.js').then((js) => {
    js.runCairoProgram(25, 60, 40);
});
