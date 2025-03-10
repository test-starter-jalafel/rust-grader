#! /bin/sh

root="/opt/test-runner"
export RUST_BACKTRACE=1

mkdir -p autograding_output

while [ $# -gt 0 ]; do
  case "$1" in
    --timeout=*)
      TIMEOUT="${1#*=}"
      ;;
    --max-score=*)
      MAX_SCORE="${1#*=}"
      MAX_SCORE="${MAX_SCORE:-0}"
      ;;
    --setup-command=*)
      SETUP_COMMAND="${1#*=}"
      ;;
    *)
      printf "***************************\n"
      printf "* Warning: Unknown argument.*\n"
      printf "***************************\n"
  esac
  shift
done

TIMEOUT=$((TIMEOUT * 60))
echo "SOMETHING IS HAPPENING!"
echo "TIMEOUT is $TIMEOUT seconds"
echo "MAX_SCORE is $MAX_SCORE"

if [ -n "$SETUP_COMMAND" ]; then
  echo "Running setup command: $SETUP_COMMAND"
  eval "$SETUP_COMMAND"
fi

# Compile the Rust file
cargo build --release --manifest-path "$root/Cargo.toml"

timeout "$TIMEOUT" "$root/target/release/run_autograding" ./ ./autograding_output/ "$MAX_SCORE" > autograding_output/results.log 2>&1
exit_status=$?
if [ $exit_status -eq 124 ]; then
  echo "The command took longer than $TIMEOUT seconds to execute. Please increase the timeout to avoid this error."
  echo '{"status": "error", "message": "The command timed out"}' > autograding_output/results.json
fi

echo "Printing results.json:"
cat autograding_output/results.json

echo "result=$(jq -c . autograding_output/results.json | jq -sRr @base64)" >> "$GITHUB_OUTPUT"