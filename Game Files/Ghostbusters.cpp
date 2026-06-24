#include "Ghostbusters.h"

static int WindowsLoop()
{
    // Score P1: 0x08E16368 (absolute virtual address)
    uint32_t scoreP1 = helpers->ReadInt32(0x08E16368, false);

    // Ghosts total P1: same address for now (user to confirm)
    uint32_t ghostsP1 = helpers->ReadInt32(0x08E16368, false);

    Outputs->SetValue(OutputHighScore, (UINT8)scoreP1);
    Outputs->SetValue(OutputCoin1, (UINT8)ghostsP1);

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

void Ghostbusters::OutputsGameLoop()
{
    if (!init)
    {
        AutoLaunchWinGame();
        Outputs = CreateOutputsFromConfig();
        m_game.name = "Ghostbusters";
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
