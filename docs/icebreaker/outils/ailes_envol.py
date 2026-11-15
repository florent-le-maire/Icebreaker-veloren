# Icebreaker : ailes dorées de l'ultime « Envol » (A.26) -> fichier MagicaVoxel.
import math, struct, sys
W, D, H = 44, 3, 30
OR, CLAIR, SOMBRE, POINTE = 10, 14, 11, 15   # indices de palette : 9 à 13 brillant, 14 à 16 lumineux (common/src/figure/cell.rs) ; 1 à 8 et 17 à 22 sont réservés
vox = {}
for dx in range(0, 21):                      # distance à la colonne
    t = dx / 20
    haut = 15 + round(13 * math.sin(t * math.pi * 0.72))
    bas = haut - (5 + round(15 * t ** 0.8))
    if dx % 4 == 3: bas += 2                 # plumes découpées
    bas = max(bas, 0)
    for z in range(bas, min(haut, H - 1) + 1):
        if z >= haut - 1: c = CLAIR          # arête du haut
        elif dx % 4 == 3: c = SOMBRE         # séparation des plumes
        elif z <= bas + 1: c = POINTE        # bout des plumes
        else: c = OR
        for x in (22 + 1 + dx, 22 - 2 - dx):
            vox[(x, 1, z)] = c
            if z >= haut - 3 and dx < 16: vox[(x, 0, z)] = c   # os de l'aile, en relief
            if z >= haut - 1 and dx < 10: vox[(x, 2, z)] = c
pal = [(0, 0, 0, 255)] * 256
pal[OR - 1] = (232, 178, 44, 255); pal[CLAIR - 1] = (255, 228, 120, 255)
pal[SOMBRE - 1] = (168, 116, 22, 255); pal[POINTE - 1] = (255, 244, 190, 255)
def chunk(nom, corps, enfants=b''): return nom + struct.pack('<II', len(corps), len(enfants)) + corps + enfants
xyzi = struct.pack('<I', len(vox)) + b''.join(struct.pack('<BBBB', x, y, z, c) for (x, y, z), c in sorted(vox.items()))
enfants = chunk(b'SIZE', struct.pack('<III', W, D, H)) + chunk(b'XYZI', xyzi) + chunk(b'RGBA', b''.join(struct.pack('<BBBB', *p) for p in pal))
open(sys.argv[1], 'wb').write(b'VOX ' + struct.pack('<I', 150) + chunk(b'MAIN', b'', enfants))
print(len(vox), 'voxels')
# aperçu de face
for z in range(H - 1, -1, -1): print(''.join({OR: '#', CLAIR: '=', SOMBRE: ':', POINTE: '.'}.get(vox.get((x, 1, z)), ' ') for x in range(W)))
