#include "SonicDashExtreme.h"
#include <fstream>

static uint32_t g_LastTicketValue = 0;
static uint32_t g_LastJackpot = 0;
static uint32_t g_LastCoin1 = 0;
static uint32_t g_LastCoin2 = 0;
static uint32_t g_LastHighScore = 0;
static bool g_Resolved = false;
static uint32_t* g_TicketAddr = nullptr;
static uint32_t g_LoopCount = 0;

static bool g_RoundActive = false;
static uint32_t g_PeakTicket = 0;
static uint32_t g_RoundNumber = 0;
static bool g_BossThisRound = false;
static char g_JsonPath[MAX_PATH] = {};

static HANDLE g_JvsMap = NULL;
static uint8_t* g_JvsData = NULL;
static uint8_t g_JvsLast[64] = {};

static void ResolveTicketPointer()
{
    __try
    {
        uintptr_t moduleBase = (uintptr_t)GetModuleHandle(NULL);
        uint32_t* ptrField = (uint32_t*)(moduleBase + 0x84A070);
        uint32_t ptrVal = *ptrField;
        if (ptrVal != 0 && ptrVal != 0xFFFFFFFF)
        {
            g_TicketAddr = (uint32_t*)(ptrVal + 8);
            char buf[256];
            sprintf_s(buf, "Ticket: ptrField=%p ptrVal=0x%08X ticketAddr=%p val=%u",
                ptrField, ptrVal, g_TicketAddr, *g_TicketAddr);
            OutputDebugStringA(buf);
        }
        else
        {
            OutputDebugStringA("Ticket: ptrField is null or invalid");
        }
    }
    __except (EXCEPTION_EXECUTE_HANDLER)
    {
        OutputDebugStringA("Ticket: AV resolving pointer chain");
    }
    g_Resolved = true;
}

static void JvsInit()
{
    g_JvsMap = OpenFileMappingA(FILE_MAP_READ, FALSE, "TeknoParrot_JvsState");
    if (g_JvsMap)
    {
        g_JvsData = (uint8_t*)MapViewOfFile(g_JvsMap, FILE_MAP_READ, 0, 0, 64);
        if (g_JvsData)
        {
            memcpy(g_JvsLast, g_JvsData, 64);
            OutputDebugStringA("JVS: Opened TeknoParrot_JvsState OK");
        }
        else
        {
            OutputDebugStringA("JVS: MapViewOfFile failed");
        }
    }
    else
    {
        OutputDebugStringA("JVS: OpenFileMapping failed");
    }
}

static void JvsPoll()
{
    if (!g_JvsData) return;
    __try
    {
        uint8_t current[64];
        memcpy(current, g_JvsData, 64);
        if (memcmp(current, g_JvsLast, 64) != 0)
        {
            char diff[128] = {};
            int pos = 0;
            for (int i = 0; i < 64; i++)
            {
                if (current[i] != g_JvsLast[i])
                {
                    char tmp[16];
                    sprintf_s(tmp, " %02x->%02x", g_JvsLast[i], current[i]);
                    if (pos + (int)strlen(tmp) < 120)
                    {
                        strcat_s(diff, tmp);
                        pos += (int)strlen(tmp);
                    }
                }
            }
            char buf[512];
            sprintf_s(buf, "JVS: %02x%02x%02x%02x %02x%02x%02x%02x %02x%02x%02x%02x%02x%02x%02x%02x ...%s",
                current[0], current[1], current[2], current[3],
                current[4], current[5], current[6], current[7],
                current[8], current[9], current[10], current[11],
                current[12], current[13], current[14], current[15],
                diff);
            OutputDebugStringA(buf);
            memcpy(g_JvsLast, current, 64);
        }
    }
    __except (EXCEPTION_EXECUTE_HANDLER)
    {
        OutputDebugStringA("JVS: AV reading state");
    }
}

static void WriteRoundJson(uint32_t round, uint32_t tickets, uint32_t jackpotVal, bool bossReached)
{
    if (!g_JsonPath[0])
    {
        GetModuleFileNameA(NULL, g_JsonPath, MAX_PATH);
        char* slash = strrchr(g_JsonPath, '\\');
        if (slash) slash[1] = 0;
        strcat_s(g_JsonPath, "tickets_outputblaster.json");
    }

    std::ifstream in(g_JsonPath);
    std::string content;
    std::string existing;
    if (in.good())
    {
        std::getline(in, existing, '\0');
        content = existing;
        in.close();
        if (!content.empty() && content.back() == ']')
            content.pop_back();
    }

    char entry[512];
    sprintf_s(entry, "%s{\"round\":%u,\"tickets\":%u,\"jackpot\":%u,\"boss\":%s,\"time\":\"%s\"",
        content.empty() || content == "[" ? "" : ",",
        round, tickets, jackpotVal,
        bossReached ? "true" : "false",
        __TIMESTAMP__);

    content = entry;
    content += "]";

    std::ofstream out(g_JsonPath);
    if (out.good())
    {
        out.write(content.c_str(), content.size());
        out.close();
    }

    char buf[256];
    sprintf_s(buf, "OB: Round %u ended: %u tickets (boss=%s) -> %s",
        round, tickets, bossReached ? "Y" : "N", g_JsonPath);
    OutputDebugStringA(buf);
}

static int WindowsLoop()
{
    UINT8 lampData1 = helpers->ReadByte(0x9C4B20, true);
    UINT8 lampData2 = helpers->ReadByte(0x9C4B21, true);
    UINT8 sideData = helpers->ReadByte(0x9C4B22, true);
    UINT8 wooferData = helpers->ReadByte(0x9C4B23, true);

    Outputs->SetValue(OutputLampStart, !!(lampData1 & 0x80));
    Outputs->SetValue(OutputLampLeader, !!(lampData1 & 0x40));
    Outputs->SetValue(OutputLampRed, !!(lampData1 & 0x08));
    Outputs->SetValue(OutputLampGreen, !!(lampData1 & 0x04));
    Outputs->SetValue(OutputLampBlue, !!(lampData1 & 0x02));
    Outputs->SetValue(OutputSideRed, !!(sideData & 0x08));
    Outputs->SetValue(OutputSideGreen, !!(sideData & 0x04));
    Outputs->SetValue(OutputSideBlue, !!(sideData & 0x02));
    Outputs->SetValue(OutputWooferRed, !!(wooferData & 0x08));
    Outputs->SetValue(OutputWooferGreen, !!(wooferData & 0x04));
    Outputs->SetValue(OutputWooferBlue, !!(wooferData & 0x02));
    Outputs->SetValue(OutputBillboardRed, !!(lampData2 & 0x08));
    Outputs->SetValue(OutputBillboardGreen, !!(lampData2 & 0x04));
    Outputs->SetValue(OutputBillboardBlue, !!(lampData2 & 0x02));
    Outputs->SetValue(OutputItemRed, !!(lampData1 & 0x01));
    Outputs->SetValue(OutputItemGreen, !!(sideData & 0x01));
    Outputs->SetValue(OutputItemBlue, !!(wooferData & 0x01));

    if (!g_Resolved)
    {
        ResolveTicketPointer();
    }

    if (!g_JvsMap) JvsInit();
    JvsPoll();

    uint32_t ticketNow = 0;
    if (g_TicketAddr != nullptr)
    {
        __try { ticketNow = *g_TicketAddr; }
        __except (EXCEPTION_EXECUTE_HANDLER) {}
    }

    uint32_t jackpot = helpers->ReadInt32(0x84A238, true);
    uint32_t coins1 = helpers->ReadInt32(0x78B3B8, true);
    uint32_t coins2a = helpers->ReadInt32(0x7A6788, true);
    uint32_t coins2b = helpers->ReadInt32(0x7A678C, true);
    uint32_t highScore = helpers->ReadInt32(0x84A678, true);

    Outputs->SetValue(OutputTicketCounter, (UINT8)ticketNow);
    Outputs->SetValue(OutputTicketJackpot, (UINT8)jackpot);
    Outputs->SetValue(OutputCoin1, (UINT8)coins1);
    Outputs->SetValue(OutputCoin2, (UINT8)max(coins2a, coins2b));
    Outputs->SetValue(OutputHighScore, (UINT8)highScore);

    if (ticketNow != g_LastTicketValue || jackpot != g_LastJackpot || coins1 != g_LastCoin1 || max(coins2a, coins2b) != g_LastCoin2 || highScore != g_LastHighScore)
    {
        g_LoopCount++;
        char buf[256];
        sprintf_s(buf, "OB: [%u] ticket=%u jackpot=%u coin1=%u coin2a=%u coin2b=%u high=%u",
            g_LoopCount, ticketNow, jackpot, coins1, coins2a, coins2b, highScore);
        OutputDebugStringA(buf);
        g_LastTicketValue = ticketNow;
        g_LastJackpot = jackpot;
        g_LastCoin1 = coins1;
        g_LastCoin2 = max(coins2a, coins2b);
        g_LastHighScore = highScore;
    }

    if (!g_RoundActive && ticketNow > 0)
    {
        g_RoundActive = true;
        g_PeakTicket = ticketNow;
        g_BossThisRound = false;
        g_RoundNumber++;
        char buf[256];
        sprintf_s(buf, "OB: Round %u START (ticket=%u)", g_RoundNumber, ticketNow);
        OutputDebugStringA(buf);
    }

    if (g_RoundActive)
    {
        if (ticketNow > g_PeakTicket)
        {
            uint32_t jump = ticketNow - g_PeakTicket;
            g_PeakTicket = ticketNow;
            if (jump >= 10 && !g_BossThisRound)
            {
                g_BossThisRound = true;
                char buf[256];
                sprintf_s(buf, "OB: Boss detected! Ticket jump=+%u", jump);
                OutputDebugStringA(buf);
            }
        }

        if (ticketNow == 0 && g_PeakTicket > 0)
        {
            g_RoundActive = false;
            WriteRoundJson(g_RoundNumber, g_PeakTicket, jackpot, g_BossThisRound);
        }
    }

    return 0;
}

static DWORD WINAPI OutputsAreGo(LPVOID lpParam)
{
    while (true)
    {
        WindowsLoop();
        Sleep(SleepA);
    }
}

void SonicDashExtreme::OutputsGameLoop()
{
    if (!init)
    {
        Outputs = CreateOutputsFromConfig();
        m_game.name = "Sonic Dash Extreme";
        Outputs->SetGame(m_game);
        Outputs->Initialize();
        Outputs->Attached();
        CreateThread(NULL, 0, OutputsAreGo, NULL, 0, NULL);
        while (GetMessage(&Msg1, NULL, NULL, 0))
        {
            TranslateMessage(&Msg1);
            DispatchMessage(&Msg1);
        }
        init = true;
    }
}
