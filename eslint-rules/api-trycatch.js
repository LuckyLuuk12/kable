import { ESLintUtils } from "@typescript-eslint/utils";

export const apiTryCatchRule = ESLintUtils.RuleCreator(
  () => "api must be wrapped in try/catch"
)({
  name: "api-try-catch",

  meta: {
    type: "problem",

    docs: {
      description: "Require try/catch around api calls"
    },

    fixable: "code",

    schema: [],

    messages: {
      missingTry: "api call must be inside a try/catch block"
    }
  },

  defaultOptions: [],

  create(context) {
    const sourceCode = context.sourceCode;

    function isApiCall(node) {
      return (
        node.callee.type === "MemberExpression" &&
        node.callee.object.type === "Identifier" &&
        node.callee.object.name === "api"
      );
    }

    function isInsideTry(node) {
      let p = node.parent;

      while (p) {
        if (p.type === "TryStatement") return true;
        p = p.parent;
      }

      return false;
    }

    return {
      AwaitExpression(node) {
        const call = node.argument;

        if (call.type !== "CallExpression") return;
        if (!isApiCall(call)) return;
        if (isInsideTry(node)) return;

        const parent = node.parent;

        let target = null;

        // case: await api.x()
        if (parent?.type === "ExpressionStatement") {
          target = parent;
        }

        // case: return await api.x()
        else if (
          parent?.type === "ReturnStatement"
        ) {
          target = parent;
        }

        if (!target) return;

        context.report({
          node: target,
          messageId: "missingTry",

          fix(fixer) {
            const text = sourceCode.getText(target);

            return fixer.replaceText(
              target,
              `try {\n  ${text}\n} catch (e) {\n  console.error("API call failed: "+${text}, e);\n}`
            );
          }
        });
      }
    };
  }
});