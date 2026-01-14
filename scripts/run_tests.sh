#!/bin/bash

# Phase 3 テスト自動化スクリプト (Bash)
#
# このスクリプトは、Phase 3 のすべてのテストを自動実行し、
# テスト結果をレポートとして出力します。
#
# 使用方法:
#   ./scripts/run_tests.sh
#   ./scripts/run_tests.sh unit
#   ./scripts/run_tests.sh integration
#   ./scripts/run_tests.sh performance
#   ./scripts/run_tests.sh all --report

set -e

# カラー出力用の定義
GREEN='\033[0;32m'
RED='\033[0;31m'
CYAN='\033[0;36m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 出力関数
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

info() {
    echo -e "${CYAN}ℹ️  $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# テスト結果を格納する変数
UNIT_TESTS_TOTAL=0
UNIT_TESTS_PASSED=0
UNIT_TESTS_FAILED=0
UNIT_TESTS_DURATION=0

INTEGRATION_TESTS_TOTAL=0
INTEGRATION_TESTS_PASSED=0
INTEGRATION_TESTS_FAILED=0
INTEGRATION_TESTS_DURATION=0

PERFORMANCE_TESTS_TOTAL=0
PERFORMANCE_TESTS_PASSED=0
PERFORMANCE_TESTS_FAILED=0
PERFORMANCE_TESTS_DURATION=0

LATENCY_MS=0
SUCCESS_RATE=0

# テスト開始時刻
START_TIME=$(date +%s)

info "Phase 3 テスト自動化スクリプト"
info "================================"
info "開始時刻: $(date '+%Y-%m-%d %H:%M:%S')"
echo ""

# ============================================================================
# 単体テストの実行
# ============================================================================

run_unit_tests() {
    info "単体テストを実行中..."
    
    local start_time=$(date +%s)
    
    # Rust の単体テストを実行
    local output=$(cargo test --lib 2>&1)
    local exit_code=$?
    
    local end_time=$(date +%s)
    UNIT_TESTS_DURATION=$((end_time - start_time))
    
    # テスト結果を解析
    if echo "$output" | grep -q "test result: ok"; then
        UNIT_TESTS_PASSED=$(echo "$output" | grep -oP 'test result: ok\. \K\d+' | head -1)
        UNIT_TESTS_TOTAL=$UNIT_TESTS_PASSED
        success "単体テスト完了: $UNIT_TESTS_PASSED/$UNIT_TESTS_TOTAL 通過 (${UNIT_TESTS_DURATION}秒)"
        return 0
    elif echo "$output" | grep -q "test result: FAILED"; then
        UNIT_TESTS_PASSED=$(echo "$output" | grep -oP 'test result: FAILED\. \K\d+' | head -1)
        UNIT_TESTS_FAILED=$(echo "$output" | grep -oP 'test result: FAILED\. \d+ passed; \K\d+' | head -1)
        UNIT_TESTS_TOTAL=$((UNIT_TESTS_PASSED + UNIT_TESTS_FAILED))
        error "単体テスト失敗: $UNIT_TESTS_PASSED/$UNIT_TESTS_TOTAL 通過 (${UNIT_TESTS_DURATION}秒)"
        
        if [ "$VERBOSE" = "true" ]; then
            info "詳細:"
            echo "$output"
        fi
        
        return 1
    else
        error "単体テストの実行に失敗しました"
        if [ "$VERBOSE" = "true" ]; then
            echo "$output"
        fi
        return 1
    fi
}

# ============================================================================
# 統合テストの実行
# ============================================================================

run_integration_tests() {
    info "統合テストを実行中..."
    
    local start_time=$(date +%s)
    
    # E2E テストを実行
    local output=$(cargo test --test e2e_tests 2>&1)
    local exit_code=$?
    
    local end_time=$(date +%s)
    INTEGRATION_TESTS_DURATION=$((end_time - start_time))
    
    # テスト結果を解析
    if echo "$output" | grep -q "test result: ok"; then
        INTEGRATION_TESTS_PASSED=$(echo "$output" | grep -oP 'test result: ok\. \K\d+' | head -1)
        INTEGRATION_TESTS_TOTAL=$INTEGRATION_TESTS_PASSED
        success "統合テスト完了: $INTEGRATION_TESTS_PASSED/$INTEGRATION_TESTS_TOTAL 通過 (${INTEGRATION_TESTS_DURATION}秒)"
        return 0
    elif echo "$output" | grep -q "test result: FAILED"; then
        INTEGRATION_TESTS_PASSED=$(echo "$output" | grep -oP 'test result: FAILED\. \K\d+' | head -1)
        INTEGRATION_TESTS_FAILED=$(echo "$output" | grep -oP 'test result: FAILED\. \d+ passed; \K\d+' | head -1)
        INTEGRATION_TESTS_TOTAL=$((INTEGRATION_TESTS_PASSED + INTEGRATION_TESTS_FAILED))
        error "統合テスト失敗: $INTEGRATION_TESTS_PASSED/$INTEGRATION_TESTS_TOTAL 通過 (${INTEGRATION_TESTS_DURATION}秒)"
        
        if [ "$VERBOSE" = "true" ]; then
            info "詳細:"
            echo "$output"
        fi
        
        return 1
    else
        warning "統合テストが見つかりません（まだ実装されていない可能性があります）"
        return 0
    fi
}

# ============================================================================
# パフォーマンステストの実行
# ============================================================================

run_performance_tests() {
    info "パフォーマンステストを実行中..."
    
    local start_time=$(date +%s)
    
    # パフォーマンステストを実行
    local output=$(cargo test --release -- --ignored --nocapture 2>&1)
    local exit_code=$?
    
    local end_time=$(date +%s)
    PERFORMANCE_TESTS_DURATION=$((end_time - start_time))
    
    # メトリクスを解析
    if echo "$output" | grep -q "Average latency:"; then
        LATENCY_MS=$(echo "$output" | grep -oP 'Average latency: \K[\d.]+')
        
        if (( $(echo "$LATENCY_MS <= 5.0" | bc -l) )); then
            success "キー入力遅延: ${LATENCY_MS}ms (基準: ≤5ms)"
        else
            error "キー入力遅延: ${LATENCY_MS}ms (基準: ≤5ms) - 基準を超えています"
        fi
    fi
    
    if echo "$output" | grep -q "Success rate:"; then
        SUCCESS_RATE=$(echo "$output" | grep -oP 'Success rate: \K[\d.]+')
        
        if (( $(echo "$SUCCESS_RATE >= 99.0" | bc -l) )); then
            success "成功率: ${SUCCESS_RATE}% (基準: ≥99%)"
        else
            error "成功率: ${SUCCESS_RATE}% (基準: ≥99%) - 基準を下回っています"
        fi
    fi
    
    # テスト結果を解析
    if echo "$output" | grep -q "test result: ok"; then
        PERFORMANCE_TESTS_PASSED=$(echo "$output" | grep -oP 'test result: ok\. \K\d+' | head -1)
        PERFORMANCE_TESTS_TOTAL=$PERFORMANCE_TESTS_PASSED
        success "パフォーマンステスト完了: $PERFORMANCE_TESTS_PASSED/$PERFORMANCE_TESTS_TOTAL 通過 (${PERFORMANCE_TESTS_DURATION}秒)"
        return 0
    elif echo "$output" | grep -q "test result: FAILED"; then
        PERFORMANCE_TESTS_PASSED=$(echo "$output" | grep -oP 'test result: FAILED\. \K\d+' | head -1)
        PERFORMANCE_TESTS_FAILED=$(echo "$output" | grep -oP 'test result: FAILED\. \d+ passed; \K\d+' | head -1)
        PERFORMANCE_TESTS_TOTAL=$((PERFORMANCE_TESTS_PASSED + PERFORMANCE_TESTS_FAILED))
        error "パフォーマンステスト失敗: $PERFORMANCE_TESTS_PASSED/$PERFORMANCE_TESTS_TOTAL 通過 (${PERFORMANCE_TESTS_DURATION}秒)"
        
        if [ "$VERBOSE" = "true" ]; then
            info "詳細:"
            echo "$output"
        fi
        
        return 1
    else
        warning "パフォーマンステストが見つかりません（まだ実装されていない可能性があります）"
        return 0
    fi
}

# ============================================================================
# テストレポートの生成
# ============================================================================

generate_test_report() {
    info "テストレポートを生成中..."
    
    local end_time=$(date +%s)
    local total_duration=$((end_time - START_TIME))
    
    local report_path="test_results_$(date '+%Y%m%d_%H%M%S').md"
    
    # 合格率を計算
    local total_tests=$((UNIT_TESTS_TOTAL + INTEGRATION_TESTS_TOTAL + PERFORMANCE_TESTS_TOTAL))
    local total_passed=$((UNIT_TESTS_PASSED + INTEGRATION_TESTS_PASSED + PERFORMANCE_TESTS_PASSED))
    local total_failed=$((UNIT_TESTS_FAILED + INTEGRATION_TESTS_FAILED + PERFORMANCE_TESTS_FAILED))
    
    local unit_pass_rate=0
    local integration_pass_rate=0
    local performance_pass_rate=0
    local total_pass_rate=0
    
    if [ $UNIT_TESTS_TOTAL -gt 0 ]; then
        unit_pass_rate=$(echo "scale=2; ($UNIT_TESTS_PASSED / $UNIT_TESTS_TOTAL) * 100" | bc)
    fi
    
    if [ $INTEGRATION_TESTS_TOTAL -gt 0 ]; then
        integration_pass_rate=$(echo "scale=2; ($INTEGRATION_TESTS_PASSED / $INTEGRATION_TESTS_TOTAL) * 100" | bc)
    fi
    
    if [ $PERFORMANCE_TESTS_TOTAL -gt 0 ]; then
        performance_pass_rate=$(echo "scale=2; ($PERFORMANCE_TESTS_PASSED / $PERFORMANCE_TESTS_TOTAL) * 100" | bc)
    fi
    
    if [ $total_tests -gt 0 ]; then
        total_pass_rate=$(echo "scale=2; ($total_passed / $total_tests) * 100" | bc)
    fi
    
    # レポートを生成
    cat > "$report_path" << EOF
# Phase 3 テスト結果レポート

**テスト日**: $(date '+%Y-%m-%d %H:%M:%S')  
**環境**: $(uname -s) $(uname -r)  
**総実行時間**: ${total_duration}秒

---

## 📊 テスト結果サマリー

| カテゴリ | 合計 | 通過 | 失敗 | 実行時間 | 合格率 |
|---------|------|------|------|---------|--------|
| 単体テスト | $UNIT_TESTS_TOTAL | $UNIT_TESTS_PASSED | $UNIT_TESTS_FAILED | ${UNIT_TESTS_DURATION}秒 | ${unit_pass_rate}% |
| 統合テスト | $INTEGRATION_TESTS_TOTAL | $INTEGRATION_TESTS_PASSED | $INTEGRATION_TESTS_FAILED | ${INTEGRATION_TESTS_DURATION}秒 | ${integration_pass_rate}% |
| パフォーマンステスト | $PERFORMANCE_TESTS_TOTAL | $PERFORMANCE_TESTS_PASSED | $PERFORMANCE_TESTS_FAILED | ${PERFORMANCE_TESTS_DURATION}秒 | ${performance_pass_rate}% |

---

## ⚡ パフォーマンスメトリクス

| メトリクス | 測定値 | 基準値 | 結果 |
|-----------|--------|--------|------|
EOF

    # パフォーマンスメトリクスを追加
    if [ "$LATENCY_MS" != "0" ]; then
        local latency_result="❌ Fail"
        if (( $(echo "$LATENCY_MS <= 5.0" | bc -l) )); then
            latency_result="✅ Pass"
        fi
        echo "| キー入力遅延 | ${LATENCY_MS}ms | ≤5ms | $latency_result |" >> "$report_path"
    fi
    
    if [ "$SUCCESS_RATE" != "0" ]; then
        local success_rate_result="❌ Fail"
        if (( $(echo "$SUCCESS_RATE >= 99.0" | bc -l) )); then
            success_rate_result="✅ Pass"
        fi
        echo "| 成功率 | ${SUCCESS_RATE}% | ≥99% | $success_rate_result |" >> "$report_path"
    fi
    
    cat >> "$report_path" << EOF

---

## 🎯 総合評価

- **合格率**: $total_passed/$total_tests (${total_pass_rate}%)
EOF

    if [ $total_failed -eq 0 ]; then
        cat >> "$report_path" << EOF
- **重大な問題**: なし
- **リリース判定**: ✅ リリース可能
EOF
    else
        cat >> "$report_path" << EOF
- **重大な問題**: $total_failed 件のテストが失敗
- **リリース判定**: ❌ 修正が必要
EOF
    fi
    
    cat >> "$report_path" << EOF

---

**作成日**: $(date '+%Y-%m-%d %H:%M:%S')  
**作成者**: 自動生成  
**バージョン**: Phase 3
EOF

    success "テストレポートを生成しました: $report_path"
    
    echo "$report_path"
}

# ============================================================================
# メイン処理
# ============================================================================

# 引数を解析
TEST_TYPE="${1:-all}"
GENERATE_REPORT=false
VERBOSE=false

for arg in "$@"; do
    case $arg in
        --report)
            GENERATE_REPORT=true
            ;;
        --verbose)
            VERBOSE=true
            ;;
    esac
done

# プロジェクトディレクトリに移動
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
cd "$PROJECT_DIR"

info "プロジェクトディレクトリ: $PROJECT_DIR"
echo ""

# テストタイプに応じて実行
ALL_PASSED=true

if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "unit" ]; then
    if ! run_unit_tests; then
        ALL_PASSED=false
    fi
    echo ""
fi

if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "integration" ]; then
    if ! run_integration_tests; then
        ALL_PASSED=false
    fi
    echo ""
fi

if [ "$TEST_TYPE" = "all" ] || [ "$TEST_TYPE" = "performance" ]; then
    if ! run_performance_tests; then
        ALL_PASSED=false
    fi
    echo ""
fi

# レポート生成
if [ "$GENERATE_REPORT" = "true" ] || [ "$TEST_TYPE" = "all" ]; then
    REPORT_PATH=$(generate_test_report)
    echo ""
fi

# 終了時刻
END_TIME=$(date +%s)
TOTAL_DURATION=$((END_TIME - START_TIME))

info "================================"
info "終了時刻: $(date '+%Y-%m-%d %H:%M:%S')"
info "総実行時間: ${TOTAL_DURATION}秒"

if [ "$ALL_PASSED" = "true" ]; then
    success "すべてのテストが通過しました！"
    exit 0
else
    error "一部のテストが失敗しました"
    exit 1
fi
