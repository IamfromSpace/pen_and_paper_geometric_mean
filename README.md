# Pen and Paper Geometric Mean

## This Project

This project seeks to compare approaches for estimating the geometric mean via pen and paper.

## Motivation

Our weekly trivia game asks bonus questions where the goal is to guess the closest number, like "What is the height in feet of the tallest building in the world?"
The half of the teams that are closest to the correct answer get a point.

We can make informed guesses, but almost never know the exact answer, and guesses vary among teammates.
To handle this, we use a wisdom of the crowd approach, averaging everyone's guess.

Wisdom of the crowd still works best with more information, so things like, "I know it's the Burj Khalifa," "I just saw that the space needle is 600ft," "I'm pretty sure it's under 200 floors," and "a standard floor is about 12ft," are all quite helpful.
However, we want to avoid information that's too close to a guess like "It must be over 1000ft, right?", as they anchor other people's guesses.
People get uncomfortable guessing numbers that break from consensus, but this technique relies on it.

The arithmetic mean, however, does quite poorly when the range of guesses gets very wide, as large answers dominate.
If the group guesses 10, 10, 10, and 100,000, then the answer will be 25k, even though most people thought the answer would be small.
Overestimating is also especially problematic, because teams that guessed under beat you by default.
Even if the correct answer was 12.5k, a team that guessed 1 would still beat you.
Despite a staggeringly worse error percentage, it was the better guess.

I suspect, that part of the issue of typical averaging, is that most of these questions are about things that obey a power law.
Building height, our example, does (citation needed).
If we instead use the geometric mean, we account for this.
The guesses of 10, 10, 10, and 100,000 have a geometric mean of 100, which likely better represents the group's overall perspective than 25k did.

However, the geometric mean is nontrivial to calculate.
This is a knowledge game, so we can't use a calculator, but we do have a pen and paper.
Doing logarithms accurately by hand is out of the question, so we instead need a simple procedure that makes a good enough estimate.

## Other Notes

People guess in round numbers.

## Approaches

### Log + Linear Interpolation

This approach does not require any memorization, and is no more difficult than the arithmetic mean.
We convert each guess to `[digit_count].[remaining_digits]`, then average.
We then reverse this process where if the average is `[x].[y]`, we take the first `x` digits starting with `y`.
So if the guesses are 300, 10000, 900, 70, we first convert this to 3.3, 5.1, 3.9, and 2.7.
The average is 3.75, so the final guess is 750.

However, there's an edge case: if the decimal portion rounds to 0 or is very small (less than 0.1), we treat it as 0.1 instead.
This prevents nonsensical answers like "0500" or "0000".
For example, with guesses 80, 80, 80, 800, we convert to 2.8, 2.8, 2.8, 3.8.
The average is 3.05, but since 0.05 < 0.1, we treat it as 3.1, giving us a final answer of 100.

Digit count serves as a proxy for the floor of the base-10 logarithm here.
Since it's always 1 greater, it precludes properties like log(a * b) = log(a) + log(b).
However, the additional 1 to all guesses just acts as an additional 1 to the average, and it disappears in the reverse process.

The whole number portion is exactly correct in initial conversion.
The decimal portion, however, is a linear interpolation between magnitudes.
This method assumes that 500 is 50% between 100 and 1000, even though it's closer to 70%.

Error from averaging the whole number portion will also result in skew.
Say guesses are 10, 10, and 100.
In this approach: 2.1, 2.1, 3.1, for an average of 2.43, and a final guess of 43.
That leftover 0.33 from averaging the whole parts (2, 2, and 3), was accurate on the log scale, but it is inaccurately converted back to the linear scale.
The true geometric mean was 215.

However, because only the decimal portion results in estimation, error is bounded.
And when all guesses have the exact same number of digits, this is simply the arithmetic mean.

### 10^(1/10) Tables

This approach requires memorization and lossy table conversions.

| digits | starts with |
|--------|-------------|
| 0.0    | 1           |
| 0.1    | 1.25        |
| 0.2    | 1.6         |
| 0.3    | 2           |
| 0.4    | 2.5         |
| 0.5    | 3           |
| 0.6    | 4           |
| 0.7    | 5           |
| 0.8    | 6           |
| 0.9    | 8           |

When converting a guess to its logarithm, we count 'zeroes' (non-decimal digits - 1) for the whole part, and then we use the table to find its decimal part.
We use the multiplier side of the table to look at what the guess starts with, and then its partial digit count to assign the decimal count.
So 2,000 becomes 3.3, 50 becomes 1.7, 1.25M becomes 6.1 and so on.
When we're in the middle, we round down (though, it's okay to interpolate if we're feeling brave).
So 350 becomes 2.5, 1,400 becomes 3.1, 11 becomes 1, and 9001 becomes 3.9.

We average these logarithmic estimates, and then use the table in reverse to reverse the process.
We use the whole part to give us the number of zeroes, and then we look up the fractional digits to find what the number starts with.
So 3.6 becomes 4,000, 2.8 becomes 600, 7.2 becomes 16M, and 4.4 becomes 25k.
If we're in the middle, we round up (or again, interpolate if we're feeling brave).
So 2.333 becomes 250, 7.75 becomes 60M, 4.167 becomes 16k.

### Comparison

We use Monte Carlo simulation to get reasonable estimates about accuracy.
To ensure correctness while maximizing speed, we use the Rust programming language for the simulation.
While a language like Haskell can frequently give us even greater correctness guarantees, logarithms cannot be calculated with perfect precision in any language.

We use LEAN to formally prove error bounds for the pen-and-paper methods.
