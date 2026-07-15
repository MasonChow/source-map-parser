#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import process from 'node:process';

const PACKAGE_NAME = 'source_map_parser_node';
const INSTALL_SCRIPT_PATH = fileURLToPath(new URL('./install.sh', import.meta.url));

function printHelp() {
  console.log(`source-map-parser CLI

Usage:
  source-map-parser update [--version <version>] [--from npm|github]
  source-map-parser --help

Commands:
  update    Update the global source_map_parser_node CLI/package.

Options:
  --version <version>   Install a specific npm version or GitHub tag. Defaults to latest.
  --from <npm|github>   Update through npm or the GitHub install script. Defaults to npm.
`);
}

function readOptionValue(argv, index, optionName) {
  const value = argv[index + 1];
  if (!value || value.startsWith('--')) {
    throw new Error(`Missing value for ${optionName}`);
  }
  return value;
}

function parseArgs(argv) {
  const args = { command: argv[2], version: 'latest', from: 'npm', help: false };
  for (let i = 3; i < argv.length; i += 1) {
    const value = argv[i];
    if (value === '--version') {
      args.version = readOptionValue(argv, i, '--version');
      i += 1;
    } else if (value === '--from') {
      args.from = readOptionValue(argv, i, '--from');
      i += 1;
    } else if (value === '--help' || value === '-h') {
      args.help = true;
    } else {
      throw new Error(`Unknown argument: ${value}`);
    }
  }
  return args;
}

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { stdio: 'inherit', ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function updateFromNpm(version) {
  const npmCommand = process.platform === 'win32' ? 'npm.cmd' : 'npm';
  const spec = version === 'latest' ? PACKAGE_NAME : `${PACKAGE_NAME}@${version}`;
  run(npmCommand, ['install', '--global', spec]);
}

function updateFromGitHub(version) {
  if (process.platform === 'win32') {
    console.error('GitHub install script update requires bash. Use: source-map-parser update --from npm');
    process.exit(1);
  }
  run('bash', [INSTALL_SCRIPT_PATH, '--version', version]);
}

try {
  const args = parseArgs(process.argv);
  if (!args.command || args.help || args.command === '--help' || args.command === '-h') {
    printHelp();
    process.exit(0);
  }
  if (args.command !== 'update') throw new Error(`Unknown command: ${args.command}`);
  if (!['npm', 'github'].includes(args.from)) throw new Error('--from must be npm or github');
  if (args.from === 'github') updateFromGitHub(args.version);
  else updateFromNpm(args.version);
} catch (error) {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
}
