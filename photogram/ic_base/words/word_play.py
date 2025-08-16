#!/usr/bin/env python3

# If we drop words ending in 's'
# 724 yields 100 five-letter words
# 1719 yields 200 five-letter words, 171 most 4 letter matches, 95 most 3 letter matches
# 1900 yields 218 five-letter words, 184 most 4 letter matches, 100 most 3 letter matches
# 2082 yields 237 five-letter words, 200 most 4 letter matches, 107 most 3 letter matches
# all yields 317 five-letter words, 260 most 4 letter matches, 128 most 3 letter matches

# If we dont drop words ending in s
# 1390 yields 211 five-letter words, 177 most 4 letter matches, 100 most 3 letter matches
# 1900 yields 277 five-letter words, 224 most 4 letter matches, 120 most 3 letter matches
# all yields 419 five-letter words, 326 most 4 letter matches, 151 most 3 letter matches

# If we dont drop words ending in s
# 1000 yields 140 six-letter words, 134 most 5 letter matches, ? most 4 letter matches
# 1760 yields 241 six-letter words, 230 most 5 letter matches, 200 most 4 letter matches
# 2000 yields 269 six-letter words, 258 most 5 letter matches, 225 most 4 letter matches
# all yields 437 six-letter words, 416 most 5 letter matches, 352 most 4 letter matches
max_hfw_words = 1390

# Downloaded from https://github.com/kloge/The-English-Open-Word-List/blob/master/EOWL%20LF%20Delimited%20Format.zip
with  open("eowl.txt") as f:
    eowl_l = f.readlines()
    pass
eowl = {}

for s in eowl_l:
    s = s.strip()
    if s != "":
        eowl[s] = 1
        pass
    pass

hfw_words = []
with open("high_frequency_words.txt") as f:
    for l in f.readlines():
        for w in l.split():
            w = w.strip()
            hfw_words.append(w)
            pass
        pass
    pass

print(len(hfw_words))


hfw = {}
for w in hfw_words[0:max_hfw_words]:
    if w != "":
        hfw[w] = 1
        pass
    pass

hfw = list(hfw.keys())
# print(hfw)

intersection = []
for w in hfw:
    if w in eowl:
        intersection.append(w)
        pass
    pass
# print(eowl)
# print(intersection)

fives = []
for i in intersection:
    if len(i)==5: # Change for 5/6 letter words
        fives.append(i)
        pass
    pass

# Build list that
not_one_letter_different = []
for f in fives:
    allowed = True
    for g in not_one_letter_different:
        same = 0
        for i in range(5):
            if f[i]==g[i]:
                same += 1
                pass
            pass
        if same >= 3: # Change for 3/4/5 letters the same
            allowed = False
            break
        pass
    if allowed:
        not_one_letter_different.append(f)
        pass
    pass


fives.sort()
print(fives)
print(len(fives))


result = not_one_letter_different
result.sort()
print(result)
print(len(result))

split_result = [[], []]
for i in range(len(result)):
    split_result[i&1].append(result[i])
    pass
def print_result(x):
    print("const WORDS : [&str; %d] = ["%len(x),
          ", ".join([('"%s"'%w) for w in x]),
          "];")
    pass
print_result(split_result[0])
print_result(split_result[1])
