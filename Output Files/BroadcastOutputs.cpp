/*This file is part of Output Blaster.

Output Blaster is free software : you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Output Blaster is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Output Blaster.If not, see < https://www.gnu.org/licenses/>.*/

#include "BroadcastOutputs.h"

CBroadcastOutputs::CBroadcastOutputs(COutputs* a, COutputs* b)
    : m_a(a), m_b(b)
{
}

CBroadcastOutputs::~CBroadcastOutputs()
{
    delete m_a;
    delete m_b;
}

bool CBroadcastOutputs::Initialize()
{
    bool ok = true;
    if (m_a) ok = m_a->Initialize() && ok;
    if (m_b) ok = m_b->Initialize() && ok;
    return ok;
}

void CBroadcastOutputs::Attached()
{
    if (m_a) m_a->Attached();
    if (m_b) m_b->Attached();
}

void CBroadcastOutputs::SendOutput(EOutputs output, UINT32 prevValue, UINT32 value)
{
    if (m_a) m_a->SetValue(output, value);
    if (m_b) m_b->SetValue(output, value);
}
