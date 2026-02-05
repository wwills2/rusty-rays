import { execSync } from 'node:child_process';

// component name = 3rd argument
const component = process.argv[2];

if (!component) {
  console.error('❌ Please specify a component, e.g. npm run ui-add button');
  process.exit(1);
}

try {
  execSync(
    `npx shadcn@latest add --path ./src/renderer/retro-ui-lib @retroui/${component}`,
    { stdio: 'inherit' },
  );
} catch {
  process.exit(1);
}
