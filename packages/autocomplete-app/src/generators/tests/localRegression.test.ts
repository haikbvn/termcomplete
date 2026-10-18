import { afterEach, expect, it, vi } from "vitest";
import logger from "loglevel";
import { executeCommandTimeout } from "@aws/amazon-q-developer-cli-api-bindings-wrappers";
import type { ArgumentParserResult } from "@aws/amazon-q-developer-cli-autocomplete-parser";
import { getScriptSuggestions } from "../scriptSuggestionsGenerator";
import { createGeneratorState } from "../../state/generators";
import type { AutocompleteState } from "../../state/types";

vi.mock(
  "@aws/amazon-q-developer-cli-api-bindings-wrappers",
  async (original) => ({
    ...(await original<object>()),
    executeCommandTimeout: vi.fn(),
  }),
);

const context = {
  annotations: [],
  tokenArray: ["git", "checkout", ""],
  currentWorkingDirectory: "/tmp/local-repository",
  currentProcess: "zsh",
  sshPrefix: "",
  searchTerm: "",
  environmentVariables: {},
};

afterEach(() => vi.restoreAllMocks());

it("keeps local generator output and its Unicode names without credentials", async () => {
  vi.mocked(executeCommandTimeout).mockResolvedValue({
    status: 0,
    stdout: "main\ntính-năng\n",
    stderr: "",
  });
  const result = await getScriptSuggestions(
    { script: ["git", "branch", "--format=%(refname:short)"], splitOn: "\n" },
    context,
    5000,
  );
  expect(result.map((item) => item.name)).toEqual(["main", "tính-năng"]);
  expect(executeCommandTimeout).toHaveBeenLastCalledWith(
    {
      command: "git",
      args: ["branch", "--format=%(refname:short)"],
      cwd: context.currentWorkingDirectory,
    },
    5000,
  );
});

it.each([
  "tool missing: ENOENT",
  "offline tool request failed",
  "generator timed out",
])("returns no suggestions when a generator fails: %s", async (message) => {
  vi.spyOn(logger, "error").mockImplementation(() => {});
  vi.mocked(executeCommandTimeout).mockRejectedValue(new Error(message));
  expect(
    await getScriptSuggestions(
      { script: ["unavailable-tool"], splitOn: "\n" },
      context,
      50,
    ),
  ).toEqual([]);
});

it("ignores an old generator result that arrives after a newer command context", async () => {
  let first!: (result: Fig.Suggestion[]) => void;
  let second!: (result: Fig.Suggestion[]) => void;
  const custom = vi
    .fn()
    .mockImplementationOnce(
      () =>
        new Promise<Fig.Suggestion[]>((resolve) => {
          first = resolve;
        }),
    )
    .mockImplementationOnce(
      () =>
        new Promise<Fig.Suggestion[]>((resolve) => {
          second = resolve;
        }),
    );
  const currentArg = { generators: [{ custom, trigger: { on: "change" } }] };
  const parsed = (searchTerm: string) =>
    ({
      currentArg,
      searchTerm,
      annotations: [],
      commandIndex: 0,
    }) as unknown as ArgumentParserResult;
  let state = {
    figState: {
      cwd: context.currentWorkingDirectory,
      processUserIsIn: "zsh",
      environmentVariables: {},
    },
    parserResult: parsed(""),
    generatorStates: [],
    command: null,
  } as unknown as AutocompleteState;
  const { triggerGenerators } = createGeneratorState(
    (_name, update) => {
      state = {
        ...state,
        ...(typeof update === "function" ? update(state) : update),
      };
    },
    () => state,
  );
  state.generatorStates = triggerGenerators(parsed("old"));
  state.parserResult = parsed("old");
  state.generatorStates = triggerGenerators(parsed("new"));
  state.parserResult = parsed("new");
  second([{ name: "new-result" }]);
  await vi.waitFor(() =>
    expect(state.generatorStates[0].result[0]?.name).toBe("new-result"),
  );
  first([{ name: "stale-result" }]);
  await new Promise((resolve) => setTimeout(resolve, 0));
  expect(state.generatorStates[0].result.map((item) => item.name)).toEqual([
    "new-result",
  ]);
});
