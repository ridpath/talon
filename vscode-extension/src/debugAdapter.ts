import {
    DebugSession,
    InitializedEvent,
    TerminatedEvent,
    StoppedEvent,
    BreakpointEvent,
    OutputEvent,
    Thread,
    StackFrame,
    Scope,
    Source,
    Handles,
    Breakpoint
} from '@vscode/debugadapter';
import { DebugProtocol } from '@vscode/debugprotocol';
import * as path from 'path';
import * as fs from 'fs';
import { spawn, ChildProcess } from 'child_process';

interface TalonLaunchRequestArguments extends DebugProtocol.LaunchRequestArguments {
    program: string;
    args?: string[];
    cwd?: string;
    env?: { [key: string]: string };
    stopOnEntry?: boolean;
    gdbPath?: string;
}

interface GDBBreakpoint {
    id: number;
    address: string;
    line: number;
    verified: boolean;
}

export class TalonDebugSession extends DebugSession {
    private static THREAD_ID = 1;
    
    private gdbProcess: ChildProcess | null = null;
    private variableHandles = new Handles<string>();
    private breakpoints = new Map<string, GDBBreakpoint[]>();
    private currentLine: number = 0;
    private registers: Map<string, string> = new Map();
    private memory: Map<string, Uint8Array> = new Map();
    private sourceFile: string = '';

    public constructor() {
        super();
        this.setDebuggerLinesStartAt1(true);
        this.setDebuggerColumnsStartAt1(true);
    }

    protected initializeRequest(
        response: DebugProtocol.InitializeResponse,
        args: DebugProtocol.InitializeRequestArguments
    ): void {
        response.body = response.body || {};
        response.body.supportsConfigurationDoneRequest = true;
        response.body.supportsEvaluateForHovers = true;
        response.body.supportsStepBack = false;
        response.body.supportsSetVariable = true;
        response.body.supportsRestartFrame = false;
        response.body.supportsGotoTargetsRequest = false;
        response.body.supportsStepInTargetsRequest = false;
        response.body.supportsCompletionsRequest = false;
        response.body.supportsModulesRequest = false;
        response.body.supportsRestartRequest = false;
        response.body.supportsExceptionOptions = false;
        response.body.supportsValueFormattingOptions = true;
        response.body.supportsExceptionInfoRequest = false;
        response.body.supportTerminateDebuggee = true;
        response.body.supportsDelayedStackTraceLoading = false;
        response.body.supportsLoadedSourcesRequest = false;
        response.body.supportsReadMemoryRequest = true;
        response.body.supportsDisassembleRequest = true;

        this.sendResponse(response);
        this.sendEvent(new InitializedEvent());
    }

    protected async launchRequest(
        response: DebugProtocol.LaunchResponse,
        args: TalonLaunchRequestArguments
    ): Promise<void> {
        this.sourceFile = args.program;
        
        const gdbPath = args.gdbPath || 'gdb';
        const compiledBinary = this.compileTalonScript(args.program);
        
        if (!compiledBinary) {
            this.sendErrorResponse(response, {
                id: 1,
                format: 'Failed to compile TALON script',
                showUser: true
            });
            return;
        }

        this.gdbProcess = spawn(gdbPath, [
            '--interpreter=mi',
            '--quiet',
            compiledBinary
        ], {
            cwd: args.cwd || path.dirname(args.program),
            env: args.env
        });

        if (!this.gdbProcess.stdout || !this.gdbProcess.stdin) {
            this.sendErrorResponse(response, {
                id: 2,
                format: 'Failed to start GDB process',
                showUser: true
            });
            return;
        }

        this.gdbProcess.stdout.on('data', (data: Buffer) => {
            this.handleGDBOutput(data.toString());
        });

        this.gdbProcess.stderr?.on('data', (data: Buffer) => {
            this.sendEvent(new OutputEvent(data.toString(), 'stderr'));
        });

        this.gdbProcess.on('exit', (code) => {
            this.sendEvent(new TerminatedEvent());
        });

        await this.sendGDBCommand('-gdb-set mi-async on');
        await this.sendGDBCommand('-file-exec-and-symbols ' + compiledBinary);

        if (args.stopOnEntry) {
            await this.sendGDBCommand('-break-insert main');
        }

        this.sendResponse(response);
    }

    protected async setBreakPointsRequest(
        response: DebugProtocol.SetBreakpointsResponse,
        args: DebugProtocol.SetBreakpointsArguments
    ): Promise<void> {
        const path = args.source.path as string;
        const clientLines = args.lines || [];

        const breakpoints: Breakpoint[] = [];

        for (const line of clientLines) {
            const result = await this.sendGDBCommand(`-break-insert ${path}:${line}`);
            
            if (result.includes('done')) {
                const bpId = this.parseBreakpointId(result);
                breakpoints.push({
                    verified: true,
                    line: line,
                    id: bpId
                });
            } else {
                breakpoints.push({
                    verified: false,
                    line: line
                });
            }
        }

        response.body = {
            breakpoints: breakpoints
        };
        this.sendResponse(response);
    }

    protected async continueRequest(
        response: DebugProtocol.ContinueResponse,
        args: DebugProtocol.ContinueArguments
    ): Promise<void> {
        await this.sendGDBCommand('-exec-continue');
        this.sendResponse(response);
    }

    protected async nextRequest(
        response: DebugProtocol.NextResponse,
        args: DebugProtocol.NextArguments
    ): Promise<void> {
        await this.sendGDBCommand('-exec-next');
        this.sendResponse(response);
    }

    protected async stepInRequest(
        response: DebugProtocol.StepInResponse,
        args: DebugProtocol.StepInArguments
    ): Promise<void> {
        await this.sendGDBCommand('-exec-step');
        this.sendResponse(response);
    }

    protected async stepOutRequest(
        response: DebugProtocol.StepOutResponse,
        args: DebugProtocol.StepOutArguments
    ): Promise<void> {
        await this.sendGDBCommand('-exec-finish');
        this.sendResponse(response);
    }

    protected threadsRequest(response: DebugProtocol.ThreadsResponse): void {
        response.body = {
            threads: [
                new Thread(TalonDebugSession.THREAD_ID, 'Main Thread')
            ]
        };
        this.sendResponse(response);
    }

    protected async stackTraceRequest(
        response: DebugProtocol.StackTraceResponse,
        args: DebugProtocol.StackTraceArguments
    ): Promise<void> {
        const frames = await this.getStackFrames();
        
        response.body = {
            stackFrames: frames,
            totalFrames: frames.length
        };
        this.sendResponse(response);
    }

    protected scopesRequest(
        response: DebugProtocol.ScopesResponse,
        args: DebugProtocol.ScopesArguments
    ): void {
        response.body = {
            scopes: [
                new Scope('Registers', this.variableHandles.create('registers'), false),
                new Scope('Local', this.variableHandles.create('local'), false),
                new Scope('Memory', this.variableHandles.create('memory'), false)
            ]
        };
        this.sendResponse(response);
    }

    protected async variablesRequest(
        response: DebugProtocol.VariablesResponse,
        args: DebugProtocol.VariablesArguments
    ): Promise<void> {
        const id = this.variableHandles.get(args.variablesReference);
        const variables: DebugProtocol.Variable[] = [];

        if (id === 'registers') {
            await this.updateRegisters();
            this.registers.forEach((value, name) => {
                variables.push({
                    name: name,
                    value: value,
                    variablesReference: 0
                });
            });
        } else if (id === 'local') {
            const locals = await this.getLocalVariables();
            variables.push(...locals);
        } else if (id === 'memory') {
            variables.push({
                name: 'Stack',
                value: '[Click to view]',
                variablesReference: 0
            });
        }

        response.body = {
            variables: variables
        };
        this.sendResponse(response);
    }

    protected async evaluateRequest(
        response: DebugProtocol.EvaluateResponse,
        args: DebugProtocol.EvaluateArguments
    ): Promise<void> {
        let result = '';

        if (args.expression.startsWith('0x')) {
            const addr = args.expression;
            const memResult = await this.sendGDBCommand(`-data-read-memory ${addr} x 1 1 8`);
            result = this.parseMemoryValue(memResult);
        } else {
            const evalResult = await this.sendGDBCommand(`-data-evaluate-expression ${args.expression}`);
            result = this.parseEvalValue(evalResult);
        }

        response.body = {
            result: result,
            variablesReference: 0
        };
        this.sendResponse(response);
    }

    protected async readMemoryRequest(
        response: DebugProtocol.ReadMemoryResponse,
        args: DebugProtocol.ReadMemoryArguments
    ): Promise<void> {
        const address = args.memoryReference;
        const count = args.count || 256;
        
        const memResult = await this.sendGDBCommand(`-data-read-memory ${address} x 1 1 ${count}`);
        const data = this.parseMemoryData(memResult);

        response.body = {
            address: address,
            data: Buffer.from(data).toString('base64')
        };
        this.sendResponse(response);
    }

    protected disconnectRequest(
        response: DebugProtocol.DisconnectResponse,
        args: DebugProtocol.DisconnectArguments
    ): void {
        if (this.gdbProcess) {
            this.gdbProcess.kill();
            this.gdbProcess = null;
        }
        this.sendResponse(response);
    }

    private compileTalonScript(scriptPath: string): string | null {
        const outputDir = path.join(path.dirname(scriptPath), '.talon_debug');
        if (!fs.existsSync(outputDir)) {
            fs.mkdirSync(outputDir, { recursive: true });
        }

        const outputBinary = path.join(outputDir, 'debug_binary');
        
        const { execSync } = require('child_process');
        try {
            execSync(`talon build "${scriptPath}" -o "${outputBinary}"`, {
                stdio: 'inherit'
            });
            return outputBinary;
        } catch (error) {
            this.sendEvent(new OutputEvent(`Compilation failed: ${error}\n`, 'stderr'));
            return null;
        }
    }

    private async sendGDBCommand(command: string): Promise<string> {
        return new Promise((resolve) => {
            if (!this.gdbProcess?.stdin) {
                resolve('');
                return;
            }

            let output = '';
            const handler = (data: Buffer) => {
                output += data.toString();
                if (output.includes('(gdb)')) {
                    this.gdbProcess?.stdout?.removeListener('data', handler);
                    resolve(output);
                }
            };

            this.gdbProcess.stdout?.on('data', handler);
            this.gdbProcess.stdin.write(command + '\n');
        });
    }

    private handleGDBOutput(output: string): void {
        if (output.includes('*stopped')) {
            const reason = this.parseStopReason(output);
            this.sendEvent(new StoppedEvent(reason, TalonDebugSession.THREAD_ID));
        } else if (output.includes('*running')) {
        } else if (output.includes('breakpoint-created')) {
            const bpId = this.parseBreakpointId(output);
            this.sendEvent(new BreakpointEvent('new', { verified: true, id: bpId }));
        }
    }

    private parseStopReason(output: string): string {
        if (output.includes('breakpoint-hit')) return 'breakpoint';
        if (output.includes('end-stepping-range')) return 'step';
        if (output.includes('signal-received')) return 'exception';
        return 'pause';
    }

    private parseBreakpointId(output: string): number {
        const match = output.match(/bkpt=\{number="(\d+)"/);
        return match ? parseInt(match[1]) : 0;
    }

    private parseMemoryValue(output: string): string {
        const match = output.match(/value="(.+?)"/);
        return match ? match[1] : '0x00';
    }

    private parseEvalValue(output: string): string {
        const match = output.match(/value="(.+?)"/);
        return match ? match[1] : '';
    }

    private parseMemoryData(output: string): Uint8Array {
        const matches = output.matchAll(/0x([0-9a-fA-F]{2})/g);
        const bytes: number[] = [];
        for (const match of matches) {
            bytes.push(parseInt(match[1], 16));
        }
        return new Uint8Array(bytes);
    }

    private async getStackFrames(): Promise<StackFrame[]> {
        const result = await this.sendGDBCommand('-stack-list-frames');
        const frames: StackFrame[] = [];

        const frameMatches = result.matchAll(/frame=\{level="(\d+)",addr="(0x[0-9a-fA-F]+)",func="(.+?)",file="(.+?)",fullname="(.+?)",line="(\d+)"/g);
        
        for (const match of frameMatches) {
            const [, level, addr, func, file, fullname, line] = match;
            frames.push(new StackFrame(
                parseInt(level),
                `${func} @ ${addr}`,
                new Source(path.basename(fullname), fullname),
                parseInt(line),
                0
            ));
        }

        return frames;
    }

    private async updateRegisters(): Promise<void> {
        const result = await this.sendGDBCommand('-data-list-register-values x');
        
        const regMatches = result.matchAll(/\{number="(\d+)",value="(0x[0-9a-fA-F]+)"\}/g);
        const regNames = ['rax', 'rbx', 'rcx', 'rdx', 'rsi', 'rdi', 'rbp', 'rsp', 
                          'r8', 'r9', 'r10', 'r11', 'r12', 'r13', 'r14', 'r15', 'rip'];
        
        let index = 0;
        for (const match of regMatches) {
            const [, , value] = match;
            if (index < regNames.length) {
                this.registers.set(regNames[index], value);
            }
            index++;
        }
    }

    private async getLocalVariables(): Promise<DebugProtocol.Variable[]> {
        const result = await this.sendGDBCommand('-stack-list-variables --simple-values');
        const variables: DebugProtocol.Variable[] = [];

        const varMatches = result.matchAll(/name="(.+?)",value="(.+?)"/g);
        for (const match of varMatches) {
            const [, name, value] = match;
            variables.push({
                name: name,
                value: value,
                variablesReference: 0
            });
        }

        return variables;
    }
}
